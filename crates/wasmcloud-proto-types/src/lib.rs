// Include the `items` module, which is generated from items.proto.
// It is important to maintain the same structure as in the proto.
pub mod generated {
    pub mod ctl {
        include!(concat!(env!("OUT_DIR"), "/wasmcloud.ctl.rs"));
    }
    // Each .proto should be included as its own module for organization purposes.
}

#[cfg(test)]
mod test {
    use crate::generated::ctl::{
        start_server, ControlInterfaceServiceClient, ControlInterfaceServiceClientPrefix,
        ControlInterfaceServiceServer, ScaleComponentRequest, ScaleComponentResponse,
    };

    struct Host;
    impl ControlInterfaceServiceServer for Host {
        fn subject_prefix(&self) -> &'static str {
            "wasmbus.ctl.vproto.default"
        }
        async fn scale_component(
            &self,
            request: ScaleComponentRequest,
        ) -> anyhow::Result<ScaleComponentResponse> {
            let component_id = format!("{}-{}", request.component_ref, request.max_instances);
            Ok(ScaleComponentResponse {
                component_id,
                success: true,
                message: "everything went smoothly and we're going to be okay".to_string(),
            })
        }
    }
    impl ControlInterfaceServiceClientPrefix for async_nats::Client {
        fn subject_prefix(&self) -> &'static str {
            "wasmbus.ctl.vproto.default"
        }
    }

    // Test and validate host functionality without NATS
    #[tokio::test]
    async fn unit_test_proto() {
        let host = Host {};
        let request = ScaleComponentRequest {
            component_ref: "test".to_string(),
            max_instances: 1,
            ..Default::default()
        };
        let response = host
            .scale_component(request)
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
            start_server(host, nats_client)
                .await
                .expect("to subscribe and start server")
        });

        let reply = nats_client
            .scale_component(ScaleComponentRequest {
                component_ref: "test".to_string(),
                max_instances: 1,
                ..Default::default()
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
