use wasmcloud_component::http;

struct Component;

http::export!(Component);

impl http::Server for Component {
    fn handle(
        _request: http::IncomingRequest,
    ) -> http::Result<http::Response<impl http::OutgoingBody>> {
        wasmcloud_component::info!("hai");
        Ok(http::Response::new("Hello from Rust!\n"))
    }
}
