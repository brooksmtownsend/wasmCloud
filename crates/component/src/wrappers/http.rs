//! This module provides utilities for writing HTTP servers and clients using the WASI HTTP API.
//!
//! It's inspired by the WASI 0.3 proposal for <https://github.com/WebAssembly/wasi-http> and will
//! be supported until the release of wasi:http@0.3.0. After that, this module will be deprecated.

use std::io::Write;

use wasi::http::{
    outgoing_handler::ErrorCode,
    types::{Fields, Method, OutgoingBody, OutgoingRequest, OutgoingResponse, ResponseOutparam},
};

// wasi:http/incoming-handler utilities

/// Macro to export [`wasi::exports::http::incoming_handler::Guest`] implementation for a type that
/// implements [`HttpServer`].
///
/// NOTE(brooksmtownsend): See a different implementation<https://github.com/wacker-dev/waki/blob/main/waki-macros/src/export.rs>.
/// While the code wasn't copied and this macro is different, the nice experience of the macro to wrap
/// the Guest implementation is what inspired me.
#[macro_export]
macro_rules! export {
    ($t:ty) => {
        impl ::wasi::exports::http::incoming_handler::Guest for $t {
            fn handle(
                incoming_request: ::wasi::http::types::IncomingRequest,
                response_out: ::wasi::http::types::ResponseOutparam,
            ) {
                match incoming_request.try_into() {
                    Ok(request) => match <Component as HttpServer>::handle(request) {
                        Ok(response) => response.to_outgoing_response(response_out),
                        Err(error) => {
                            ::wasi::http::types::ResponseOutparam::set(response_out, Err(error))
                        }
                    },
                    Err(e) => ::wasi::http::types::ResponseOutparam::set(response_out, Err(e)),
                }
            }
        }
        type ComponentHttpExportAlias = $t;
        ::wasi::http::proxy::export!(ComponentHttpExportAlias);
    };
}

pub use export;

pub trait HttpServer {
    fn handle(request: Request) -> Result<Response, ErrorCode>;
}

pub struct Request {
    inner: wasi::http::types::IncomingRequest,

    /// The body can either be an input stream or an output stream. On an incoming request, the body
    /// will be an input stream. On an outgoing request, the body will be an output stream.
    body: wasi::http::types::InputStream,
    /// Must be held to keep the request alive
    #[allow(unused)]
    incoming_body: wasi::http::types::IncomingBody,
}

impl TryFrom<wasi::http::types::IncomingRequest> for Request {
    type Error = ErrorCode;

    fn try_from(inner: wasi::http::types::IncomingRequest) -> Result<Self, Self::Error> {
        let incoming_body = inner
            .consume()
            .map_err(|_| error_code("failed to consume incoming request"))?;
        let body = incoming_body
            .stream()
            .map_err(|_| error_code("failed to get incoming body stream from incoming request"))?;
        Ok(Self {
            inner,
            body,
            incoming_body,
        })
    }
}

impl Request {
    pub fn method(&self) -> Method {
        self.inner.method()
    }

    pub fn headers(&self) -> Fields {
        self.inner.headers()
    }

    pub fn into_inner(self) -> wasi::http::types::IncomingRequest {
        self.inner
    }
}

pub struct ResponseBuilder {
    status_code: Option<u16>,
    body: Option<Vec<u8>>,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        Self {
            status_code: None,
            body: None,
        }
    }

    pub fn status_code(mut self, status_code: u16) -> Self {
        self.status_code = Some(status_code);
        self
    }

    pub fn body(mut self, body: impl AsRef<[u8]>) -> Self {
        self.body = Some(body.as_ref().to_vec());
        self
    }

    pub fn build(self) -> Response {
        Response {
            status_code: self.status_code.unwrap_or(200),
            body: self.body.unwrap_or_default(),
        }
    }
}

pub struct Response {
    status_code: u16,
    body: Vec<u8>,
}

impl Response {
    pub fn ok(body: impl AsRef<[u8]>) -> Self {
        Self {
            status_code: 200,
            body: body.as_ref().to_vec(),
        }
    }

    pub fn to_outgoing_response(self, response_out: ResponseOutparam) {
        // Construct response, returning server errors if possible
        let response = OutgoingResponse::new(Fields::new());
        if response.set_status_code(self.status_code).is_err() {
            ResponseOutparam::set(response_out, Err(error_code("failed to set status code")));
            return;
        }
        let Ok(response_body) = response.body() else {
            ResponseOutparam::set(
                response_out,
                Err(error_code("failed to get outgoing body handle")),
            );
            return;
        };
        let Ok(mut response_write) = response_body.write() else {
            ResponseOutparam::set(
                response_out,
                Err(error_code("failed to get write handle to outgoing body")),
            );
            return;
        };

        // Set the response before writing the body. At this point, an error can't be returned.
        ResponseOutparam::set(response_out, Ok(response));
        response_write
            .write_all(&self.body)
            .expect("failed to write body stream");
        drop(response_write);
        OutgoingBody::finish(response_body, None).expect("failed to finish outgoing body");
    }
}

// wasi:http/outgoing-handler utilities

impl TryInto<OutgoingRequest> for Request {
    type Error = ErrorCode;

    fn try_into(self) -> Result<OutgoingRequest, Self::Error> {
        let headers = self.headers();
        let method = self.method();
        // Construct OutgoingRequest
        let outgoing_request = OutgoingRequest::new(headers);
        outgoing_request
            .set_method(&method)
            .map_err(|_| error_code("failed to set method on outgoing request"))?;
        let outgoing_body = outgoing_request
            .body()
            .map_err(|_| error_code("failed to get handle to outgoing body"))?;
        // Stream body to outgoing request
        let out_stream = outgoing_body
            .write()
            .map_err(|_| error_code("failed to get write handle to outgoing body"))?;
        out_stream
            .splice(&self.body, u64::MAX)
            .map_err(|_| error_code("failed to write body stream"))?;
        OutgoingBody::finish(outgoing_body, None)?;
        drop(out_stream);

        Ok(outgoing_request)
    }
}

/// Helper function to construct an internal server `ErrorCode` with an error message.
fn error_code(e: impl ToString) -> ErrorCode {
    ErrorCode::InternalError(Some(e.to_string()))
}

/// Send an outgoing HTTP request and return the response.
pub fn handle(_request: Request) -> Result<Response, ErrorCode> {
    // let resp = wasi::http::outgoing_handler::handle(request.try_into()?, None);
    unimplemented!()
}
