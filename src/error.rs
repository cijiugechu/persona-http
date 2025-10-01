use napi::{Error as NapiError, Status};

use rnet_bindings_core::Error;

pub fn to_napi_error(err: Error) -> NapiError {
  match err {
    Error::Memory => napi_error(
      Status::GenericFailure,
      "memory access error",
      "ERR_RNET_MEMORY",
    ),
    Error::StopIteration => napi_error(
      Status::GenericFailure,
      "iterator exhausted",
      "ERR_RNET_STOP_ITERATION",
    ),
    Error::StopAsyncIteration => napi_error(
      Status::GenericFailure,
      "async iterator exhausted",
      "ERR_RNET_STOP_ASYNC_ITERATION",
    ),
    Error::WebSocketDisconnected => napi_error(
      Status::GenericFailure,
      "websocket disconnected",
      "ERR_RNET_WEBSOCKET_DISCONNECTED",
    ),
    Error::InvalidHeaderName(err) => napi_error(
      Status::InvalidArg,
      format!("invalid header name: {err}"),
      "ERR_RNET_INVALID_HEADER_NAME",
    ),
    Error::InvalidHeaderValue(err) => napi_error(
      Status::InvalidArg,
      format!("invalid header value: {err}"),
      "ERR_RNET_INVALID_HEADER_VALUE",
    ),
    Error::Timeout(err) => napi_error(
      Status::GenericFailure,
      format!("operation timed out: {err}"),
      "ERR_RNET_TIMEOUT",
    ),
    Error::Builder(err) => napi_error(
      Status::GenericFailure,
      format!("failed to build request: {err}"),
      "ERR_RNET_BUILDER",
    ),
    Error::IO(err) => napi_error(
      Status::GenericFailure,
      format!("io error: {err}"),
      "ERR_RNET_IO",
    ),
    Error::Decode(err) => napi_error(
      Status::GenericFailure,
      format!("decode error: {err}"),
      "ERR_RNET_DECODE",
    ),
    Error::Library(err) => napi_error(
      Status::GenericFailure,
      format!("library error: {err}"),
      "ERR_RNET_LIBRARY",
    ),
  }
}

fn napi_error(status: Status, message: impl Into<String>, code: &'static str) -> NapiError {
  let message = message.into();
  let reason = format!("{code}: {message}");
  NapiError::new(status, reason)
}
