// Include the `items` module, which is generated from items.proto.
// It is important to maintain the same structure as in the proto.
pub mod wasmcloud {
    pub mod ctl {
        include!(concat!(env!("OUT_DIR"), "/wasmcloud.types.rs"));
    }
}

#[cfg(test)]
mod test {
    use super::wasmcloud;

    use bytes::BytesMut;
    use prost::Message;
    use tokio::task::JoinSet;

    // (Host-side) implementation of the ControlInterface
    pub trait ControlServer {
        fn start_component(
            &self,
            request: wasmcloud::ctl::StartComponentRequest,
        ) -> impl futures::Future<Output = Result<wasmcloud::ctl::StartComponentResponse, String>> + Send;
    }
    pub struct WasmcloudControlServer {}
    impl ControlServer for WasmcloudControlServer {
        async fn start_component(
            &self,
            request: wasmcloud::ctl::StartComponentRequest,
        ) -> Result<wasmcloud::ctl::StartComponentResponse, String> {
            let component_id = format!("{}-{}", request.reference, request.max_instances);
            Ok(wasmcloud::ctl::StartComponentResponse {
                component_id,
                message: "everything went smoothly and we're going to be okay".to_string(),
            })
        }
    }
    async fn start_server<S>(server: S, client: async_nats::Client) -> Result<JoinSet<()>, String>
    where
        S: ControlServer + Send + 'static,
    {
        let mut sub = client
            .subscribe("wasmbus.ctl.proto.>")
            .await
            .expect("to subscribe to start component");
        use futures::StreamExt;
        let mut tasks = JoinSet::new();
        tasks.spawn(async move {
            while let Some(message) = sub.next().await {
                if message.subject.contains("start.component") {
                    let request = wasmcloud::ctl::StartComponentRequest::decode(message.payload)
                        .expect("to decode request");
                    let reply = server
                        .start_component(request)
                        .await
                        .expect("to start component");
                    if let Some(reply_to) = message.reply {
                        let mut buf = BytesMut::with_capacity(reply.encoded_len());
                        reply
                            .encode(&mut buf)
                            .expect("to encode without reaching capacity");
                        client
                            .publish(reply_to, buf.into())
                            .await
                            .expect("to send reply");
                    }
                }
            }
        });

        Ok(tasks)
    }

    // (wash/wadm/client-side) implementation of the ControlInterface
    pub trait ControlClient {
        async fn start_component(
            &self,
            request: wasmcloud::ctl::StartComponentRequest,
        ) -> Result<wasmcloud::ctl::StartComponentResponse, String>;
    }
    pub struct WasmcloudControlClient {
        client: async_nats::Client,
    }
    impl WasmcloudControlClient {
        pub fn new(client: async_nats::Client) -> Self {
            Self { client }
        }
    }
    impl ControlClient for WasmcloudControlClient {
        async fn start_component(
            &self,
            request: wasmcloud::ctl::StartComponentRequest,
        ) -> Result<wasmcloud::ctl::StartComponentResponse, String> {
            let mut buf = BytesMut::with_capacity(request.encoded_len());
            request
                .encode(&mut buf)
                .expect("to encode without reaching capacity");
            let reply = self
                .client
                .request("wasmbus.ctl.proto.start.component", buf.into())
                .await
                .expect("to send request");
            wasmcloud::ctl::StartComponentResponse::decode(reply.payload).map_err(|e| e.to_string())
        }
    }

    #[tokio::test]
    async fn end_to_end_proto() {
        let server = WasmcloudControlServer {};
        let nats_client = async_nats::connect("nats://127.0.0.1:4222")
            .await
            .expect("should connect to NATS");
        let _task = start_server(server, nats_client)
            .await
            .expect("to start server");

        let client = WasmcloudControlClient::new(
            async_nats::connect("nats://127.0.0.1:4222")
                .await
                .expect("should connect to NATS"),
        );
        let reply = client
            .start_component(wasmcloud::ctl::StartComponentRequest {
                reference: "test".to_string(),
                max_instances: 1,
            })
            .await
            .expect("reply to be okay");

        assert_eq!(reply.component_id, "test-1");
        assert_eq!(
            reply.message,
            "everything went smoothly and we're going to be okay"
        );
    }
}
