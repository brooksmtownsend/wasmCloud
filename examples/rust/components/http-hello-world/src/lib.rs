use wasmcloud_component::http::{HttpServer, Request, Response, ResponseBuilder};
use wasmcloud_component::wasi::http::types::ErrorCode;

wasmcloud_component::http::export!(Component);

struct Component;

impl HttpServer for Component {
    fn handle(_request: Request) -> Result<Response, ErrorCode> {
        Ok(Response::ok("Hello from Rust!"))
        // Ok(ResponseBuilder::new()
        //     .status_code(400)
        //     .body("Bad request............ awkward.")
        //     .build())
    }
}
