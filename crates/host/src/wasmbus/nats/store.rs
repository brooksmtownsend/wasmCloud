//! Implementation of the [ConfigManager] trait for NATS JetStream KV [Store].

use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Context as _};
use async_nats::jetstream::kv::{Entry as KvEntry, Operation, Store};
use bytes::Bytes;
use futures::{StreamExt as _, TryStreamExt as _};
use tokio::{sync::watch, task::JoinSet};
use tracing::{debug, error, instrument, warn};

use crate::store::StoreManager;

#[async_trait::async_trait]
impl StoreManager for Store {
    #[instrument(level = "debug", skip(self))]
    async fn get(&self, key: &str) -> anyhow::Result<Option<Bytes>> {
        self.get(key)
            .await
            .map_err(|err| anyhow::anyhow!("Failed to get config: {}", err))
    }

    #[instrument(level = "debug", skip(self, value))]
    async fn put(&self, key: &str, value: Bytes) -> anyhow::Result<()> {
        self.put(key, value)
            .await
            .map(|_| ())
            .map_err(|err| anyhow::anyhow!("Failed to set config: {}", err))
    }

    #[instrument(level = "debug", skip(self))]
    async fn del(&self, key: &str) -> anyhow::Result<()> {
        self.purge(key)
            .await
            .map_err(|err| anyhow::anyhow!("Failed to delete config: {}", err))
    }
}

/// This is an extra implementation for the host to process entries coming from a JetStream bucket.
impl crate::wasmbus::Host {
    #[instrument(level = "trace", skip_all)]
    pub(crate) async fn process_entry(
        &self,
        KvEntry {
            key,
            value,
            operation,
            ..
        }: KvEntry,
    ) {
        let key_id = key.split_once('_');
        let res = match (operation, key_id) {
            (Operation::Put, Some(("COMPONENT", id))) => {
                self.process_component_spec_put(id, value).await
            }
            (Operation::Delete, Some(("COMPONENT", id))) => {
                self.process_component_spec_delete(id).await
            }
            (Operation::Put, Some(("LINKDEF", _id))) => {
                debug!("ignoring deprecated LINKDEF put operation");
                Ok(())
            }
            (Operation::Delete, Some(("LINKDEF", _id))) => {
                debug!("ignoring deprecated LINKDEF delete operation");
                Ok(())
            }
            (Operation::Put, Some(("CLAIMS", pubkey))) => {
                self.process_claims_put(pubkey, value).await
            }
            (Operation::Delete, Some(("CLAIMS", pubkey))) => {
                self.process_claims_delete(pubkey, value).await
            }
            (operation, Some(("REFMAP", id))) => {
                // TODO: process REFMAP entries
                debug!(?operation, id, "ignoring REFMAP entry");
                Ok(())
            }
            _ => {
                warn!(key, ?operation, "unsupported KV bucket entry");
                Ok(())
            }
        };
        if let Err(error) = &res {
            error!(key, ?operation, ?error, "failed to process KV bucket entry");
        }
    }
}

//TODO(brooksmtownsend): Make sure that the configbundle accomplishes this for config
/// Watch the JetStream bucket for changes to the ComponentSpec and claims data
pub async fn data_watch(
    tasks: &mut JoinSet<anyhow::Result<()>>,
    store: Store,
    host: Arc<crate::wasmbus::Host>,
) -> anyhow::Result<()> {
    tasks.spawn({
        let host = Arc::clone(&host);
        let data = store.clone();
        async move {
            // Setup data watch first
            let data_watch = data
                .watch_all()
                .await
                .context("failed to watch lattice data bucket")?;

            // Process existing data without emitting events
            data.keys()
                .await
                .context("failed to read keys of lattice data bucket")?
                .map_err(|e| anyhow!(e).context("failed to read lattice data stream"))
                .try_filter_map(|key| async {
                    data.entry(key)
                        .await
                        .context("failed to get entry in lattice data bucket")
                })
                .for_each(|entry| async {
                    match entry {
                        Ok(entry) => host.process_entry(entry).await,
                        Err(err) => {
                            error!(%err, "failed to read entry from lattice data bucket")
                        }
                    }
                })
                .await;
            // TODO(brooksmtownsend): Do we need this?
            // let mut data_watch = Abortable::new(data_watch, data_watch_abort_reg);
            data_watch
                // .by_ref()
                .for_each({
                    let host = Arc::clone(&host);
                    move |entry| {
                        let host = Arc::clone(&host);
                        async move {
                            match entry {
                                Err(error) => {
                                    error!("failed to watch lattice data bucket: {error}");
                                }
                                Ok(entry) => host.process_entry(entry).await,
                            }
                        }
                    }
                })
                .await;
            let deadline = { *host.stop_rx.borrow() };
            host.stop_tx.send_replace(deadline);
            // if data_watch.is_aborted() {
            //     info!("data watch task gracefully stopped");
            // } else {
            //     error!("data watch task unexpectedly stopped");
            // }
            Ok(())
        }
    });

    Ok(())
}

//TODO(brooksmtownsend): Reinstate this
#[allow(dead_code)]
async fn watcher_loop(
    store: Store,
    name: String,
    tx: watch::Sender<HashMap<String, String>>,
    done: tokio::sync::oneshot::Sender<anyhow::Result<()>>,
) {
    // We need to watch with history so we can get the initial config.
    let mut watcher = match store.watch(&name).await {
        Ok(watcher) => {
            done.send(Ok(())).expect(
                "Receiver for watcher setup should not have been dropped. This is programmer error",
            );
            watcher
        }
        Err(e) => {
            done.send(Err(anyhow::anyhow!(
                "Error setting up watcher for {}: {}",
                name,
                e
            )))
            .expect(
                "Receiver for watcher setup should not have been dropped. This is programmer error",
            );
            return;
        }
    };
    loop {
        match watcher.try_next().await {
            Ok(Some(entry)) if matches!(entry.operation, Operation::Delete | Operation::Purge) => {
                // NOTE(thomastaylor312): We should probably do something and notify something up
                // the chain if we get a delete or purge event of a config that is still being used.
                // For now we just zero it out
                tx.send_replace(HashMap::new());
            }
            Ok(Some(entry)) => {
                let config: HashMap<String, String> = match serde_json::from_slice(&entry.value) {
                    Ok(config) => config,
                    Err(e) => {
                        error!(%name, error = %e, "Error decoding config from store during watch");
                        continue;
                    }
                };
                tx.send_if_modified(|current| {
                    if current == &config {
                        false
                    } else {
                        *current = config;
                        true
                    }
                });
            }
            Ok(None) => {
                error!(%name, "Watcher for config has closed");
                return;
            }
            Err(e) => {
                error!(%name, error = %e, "Error reading from watcher for config. Will wait for next entry");
                continue;
            }
        }
    }
}
