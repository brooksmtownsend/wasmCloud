// Include the `items` module, which is generated from items.proto.
// It is important to maintain the same structure as in the proto.
pub mod wasmcloud {
    pub mod ctl {
        include!("generated/wasmcloud.types.rs");
    }
}

#[cfg(test)]
mod test {
    use super::wasmcloud;
    use crate::wasmcloud::ctl::{ControlInterfaceServiceClient, ControlInterfaceServiceServer};

    struct Host;
    impl ControlInterfaceServiceServer for Host {
        async fn start_component(
            &self,
            request: wasmcloud::ctl::StartComponentRequest,
        ) -> anyhow::Result<wasmcloud::ctl::StartComponentResponse> {
            let component_id = format!("{}-{}", request.reference, request.max_instances);
            Ok(wasmcloud::ctl::StartComponentResponse {
                component_id,
                message: "everything went smoothly and we're going to be okay".to_string(),
            })
        }
    }

    // Test and validate host functionality without NATS
    #[tokio::test]
    async fn unit_test_proto() {
        let host = Host {};
        let request = wasmcloud::ctl::StartComponentRequest {
            reference: "test".to_string(),
            max_instances: 1,
        };
        let response = host
            .start_component(request)
            .await
            .expect("host to handle request");

        assert_eq!(response.component_id, "test-1");
        assert_eq!(
            response.message,
            "everything went smoothly and we're going to be okay"
        );
    }

    // Test and validate host functionality with NATS
    #[tokio::test]
    async fn end_to_end_proto() {
        let nats_client = async_nats::connect("nats://127.0.0.1:4222")
            .await
            .expect("should connect to NATS");
        let host = Host {};
        let server = tokio::spawn({
            let nats_client = nats_client.clone();
            wasmcloud::ctl::start_server(host, nats_client)
                .await
                .expect("to subscribe and start server")
        });

        let reply = nats_client
            .start_component(crate::wasmcloud::ctl::StartComponentRequest {
                reference: "test".to_string(),
                max_instances: 1,
            })
            .await
            .expect("should start component");

        assert_eq!(reply.component_id, "test-1");
        assert_eq!(
            reply.message,
            "everything went smoothly and we're going to be okay"
        );

        server.abort();
    }
}
