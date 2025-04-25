//! The NATS implementation of the control interface.

use anyhow::Context as _;
use bytes::Bytes;
use futures::future::Either;
use futures::stream::SelectAll;
use futures::{Stream, StreamExt};
use nkeys::KeyPair;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tracing::{error, instrument, trace, warn};
use wasmcloud_control_interface::CtlResponse;
use wasmcloud_core::CTL_API_VERSION_1;

use crate::wasmbus::serialize_ctl_response;

#[derive(Debug)]
pub(crate) struct Queue {
    all_streams: SelectAll<async_nats::Subscriber>,
}

impl Stream for Queue {
    type Item = async_nats::Message;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.all_streams.poll_next_unpin(cx)
    }
}

impl Queue {
    #[instrument]
    pub(crate) async fn new(
        nats: &async_nats::Client,
        topic_prefix: &str,
        lattice: &str,
        host_key: &KeyPair,
        component_auction: bool,
        provider_auction: bool,
    ) -> anyhow::Result<Self> {
        let host_id = host_key.public_key();
        let mut subs = vec![
            Either::Left(nats.subscribe(format!(
                "{topic_prefix}.{CTL_API_VERSION_1}.{lattice}.registry.put",
            ))),
            Either::Left(nats.subscribe(format!(
                "{topic_prefix}.{CTL_API_VERSION_1}.{lattice}.host.ping",
            ))),
            Either::Right(nats.queue_subscribe(
                format!("{topic_prefix}.{CTL_API_VERSION_1}.{lattice}.link.*"),
                format!("{topic_prefix}.{CTL_API_VERSION_1}.{lattice}.link",),
            )),
            Either::Right(nats.queue_subscribe(
                format!("{topic_prefix}.{CTL_API_VERSION_1}.{lattice}.claims.get"),
                format!("{topic_prefix}.{CTL_API_VERSION_1}.{lattice}.claims"),
            )),
            Either::Left(nats.subscribe(format!(
                "{topic_prefix}.{CTL_API_VERSION_1}.{lattice}.component.*.{host_id}"
            ))),
            Either::Left(nats.subscribe(format!(
                "{topic_prefix}.{CTL_API_VERSION_1}.{lattice}.provider.*.{host_id}"
            ))),
            Either::Left(nats.subscribe(format!(
                "{topic_prefix}.{CTL_API_VERSION_1}.{lattice}.label.*.{host_id}"
            ))),
            Either::Left(nats.subscribe(format!(
                "{topic_prefix}.{CTL_API_VERSION_1}.{lattice}.host.*.{host_id}"
            ))),
            Either::Right(nats.queue_subscribe(
                format!("{topic_prefix}.{CTL_API_VERSION_1}.{lattice}.config.>"),
                format!("{topic_prefix}.{CTL_API_VERSION_1}.{lattice}.config"),
            )),
        ];
        if component_auction {
            subs.push(Either::Left(nats.subscribe(format!(
                "{topic_prefix}.{CTL_API_VERSION_1}.{lattice}.component.auction",
            ))));
        }
        if provider_auction {
            subs.push(Either::Left(nats.subscribe(format!(
                "{topic_prefix}.{CTL_API_VERSION_1}.{lattice}.provider.auction",
            ))));
        }
        let streams = futures::future::join_all(subs)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, async_nats::SubscribeError>>()
            .context("failed to subscribe to queues")?;
        Ok(Self {
            all_streams: futures::stream::select_all(streams),
        })
    }
}

impl crate::wasmbus::Host {
    #[instrument(level = "trace", skip_all, fields(subject = %message.subject))]
    pub(crate) async fn handle_ctl_message(
        self: Arc<Self>,
        message: async_nats::Message,
    ) -> Option<Bytes> {
        // NOTE: if log level is not `trace`, this won't have an effect, since the current span is
        // disabled. In most cases that's fine, since we aren't aware of any control interface
        // requests including a trace context
        opentelemetry_nats::attach_span_context(&message);
        // Skip the topic prefix, the version, and the lattice
        // e.g. `wasmbus.ctl.v1.{prefix}`
        let subject = message.subject;
        let mut parts = subject
            .trim()
            // TODO(brooksmtownsend): topic prefix parsing elsewhere
            // .trim_start_matches(&self.ctl_topic_prefix)
            .trim_start_matches("wasmbus.ctl")
            .trim_start_matches('.')
            .split('.')
            .skip(2);
        trace!(%subject, "handling control interface request");

        // This response is a wrapped Result<Option<Result<Vec<u8>>>> for a good reason.
        // The outer Result is for reporting protocol errors in handling the request, e.g. failing to
        //    deserialize the request payload.
        // The Option is for the case where the request is handled successfully, but the handler
        //    doesn't want to send a response back to the client, like with an auction.
        // The inner Result is purely for the success or failure of serializing the [CtlResponse], which
        //    should never fail but it's a result we must handle.
        // And finally, the Vec<u8> is the serialized [CtlResponse] that we'll send back to the client
        let ctl_response = match (parts.next(), parts.next(), parts.next(), parts.next()) {
            // Component commands
            (Some("component"), Some("auction"), None, None) => self
                .handle_auction_component(message.payload)
                .await
                .map(serialize_ctl_response),
            (Some("component"), Some("scale"), Some(_host_id), None) => Arc::clone(&self)
                .handle_scale_component(message.payload)
                .await
                .map(Some)
                .map(serialize_ctl_response),
            (Some("component"), Some("update"), Some(_host_id), None) => Arc::clone(&self)
                .handle_update_component(message.payload)
                .await
                .map(Some)
                .map(serialize_ctl_response),
            // Provider commands
            (Some("provider"), Some("auction"), None, None) => self
                .handle_auction_provider(message.payload)
                .await
                .map(serialize_ctl_response),
            (Some("provider"), Some("start"), Some(_host_id), None) => Arc::clone(&self)
                .handle_start_provider(message.payload)
                .await
                .map(serialize_ctl_response),
            (Some("provider"), Some("stop"), Some(_host_id), None) => self
                .handle_stop_provider(message.payload)
                .await
                .map(Some)
                .map(serialize_ctl_response),
            // Host commands
            (Some("host"), Some("get"), Some(_host_id), None) => self
                .handle_inventory()
                .await
                .map(Some)
                .map(serialize_ctl_response),
            (Some("host"), Some("ping"), None, None) => self
                .handle_ping_hosts()
                .await
                .map(Some)
                .map(serialize_ctl_response),
            (Some("host"), Some("stop"), Some(host_id), None) => self
                .handle_stop_host(message.payload, host_id)
                .await
                .map(Some)
                .map(serialize_ctl_response),
            // Claims commands
            (Some("claims"), Some("get"), None, None) => self
                .handle_claims()
                .await
                .map(Some)
                .map(serialize_ctl_response),
            // Link commands
            (Some("link"), Some("del"), None, None) => self
                .handle_link_del(message.payload)
                .await
                .map(Some)
                .map(serialize_ctl_response),
            (Some("link"), Some("get"), None, None) => {
                // Explicitly returning a Vec<u8> for non-cloning efficiency within handle_links
                self.handle_links().await.map(|bytes| Some(Ok(bytes)))
            }
            (Some("link"), Some("put"), None, None) => self
                .handle_link_put(message.payload)
                .await
                .map(Some)
                .map(serialize_ctl_response),
            // Label commands
            (Some("label"), Some("del"), Some(host_id), None) => self
                .handle_label_del(host_id, message.payload)
                .await
                .map(Some)
                .map(serialize_ctl_response),
            (Some("label"), Some("put"), Some(host_id), None) => self
                .handle_label_put(host_id, message.payload)
                .await
                .map(Some)
                .map(serialize_ctl_response),
            // Registry commands
            (Some("registry"), Some("put"), None, None) => self
                .handle_registries_put(message.payload)
                .await
                .map(Some)
                .map(serialize_ctl_response),
            // Config commands
            (Some("config"), Some("get"), Some(config_name), None) => self
                .handle_config_get(config_name)
                .await
                .map(|bytes| Some(Ok(bytes))),
            (Some("config"), Some("put"), Some(config_name), None) => self
                .handle_config_put(config_name, message.payload)
                .await
                .map(Some)
                .map(serialize_ctl_response),
            (Some("config"), Some("del"), Some(config_name), None) => self
                .handle_config_delete(config_name)
                .await
                .map(Some)
                .map(serialize_ctl_response),
            // Topic fallback
            _ => {
                warn!(%subject, "received control interface request on unsupported subject");
                Ok(serialize_ctl_response(Some(CtlResponse::error(
                    "unsupported subject",
                ))))
            }
        };

        if let Err(err) = &ctl_response {
            error!(%subject, ?err, "failed to handle control interface request");
        } else {
            trace!(%subject, "handled control interface request");
        }

        match ctl_response {
            Ok(Some(Ok(payload))) => Some(payload.into()),
            // No response from the host (e.g. auctioning provider)
            Ok(None) => None,
            Err(e) => Some(
                serde_json::to_vec(&CtlResponse::error(&e.to_string()))
                    .context("failed to encode control interface response")
                    // This should never fail to serialize, but the fallback ensures that we send
                    // something back to the client even if we somehow fail.
                    .unwrap_or_else(|_| format!(r#"{{"success":false,"error":"{e}"}}"#).into())
                    .into(),
            ),
            // This would only occur if we failed to serialize a valid CtlResponse. This is
            // programmer error.
            Ok(Some(Err(e))) => Some(
                serde_json::to_vec(&CtlResponse::error(&e.to_string()))
                    .context("failed to encode control interface response")
                    .unwrap_or_else(|_| format!(r#"{{"success":false,"error":"{e}"}}"#).into())
                    .into(),
            ),
        }
    }
}
