use std::fmt;

use cookie::ParseError;
use http::header;

/// Unified error enum shared across bindings.
#[derive(Debug)]
pub enum Error {
  Memory,
  StopIteration,
  StopAsyncIteration,
  WebSocketDisconnected,
  InvalidHeaderName(header::InvalidHeaderName),
  InvalidHeaderValue(header::InvalidHeaderValue),
  Timeout(tokio::time::error::Elapsed),
  Builder(http::Error),
  IO(std::io::Error),
  Decode(ParseError),
  Library(wreq::Error),
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Error::Memory => write!(f, "memory access error"),
      Error::StopIteration => write!(f, "iterator exhausted"),
      Error::StopAsyncIteration => write!(f, "async iterator exhausted"),
      Error::WebSocketDisconnected => write!(f, "websocket disconnected"),
      Error::InvalidHeaderName(err) => write!(f, "invalid header name: {err:?}"),
      Error::InvalidHeaderValue(err) => write!(f, "invalid header value: {err:?}"),
      Error::Timeout(err) => write!(f, "timeout: {err:?}"),
      Error::Builder(err) => write!(f, "builder error: {err:?}"),
      Error::IO(err) => write!(f, "io error: {err}"),
      Error::Decode(err) => write!(f, "decode error: {err}"),
      Error::Library(err) => write!(f, "library error: {err:?}"),
    }
  }
}

impl std::error::Error for Error {}

impl From<header::InvalidHeaderName> for Error {
  fn from(err: header::InvalidHeaderName) -> Self {
    Error::InvalidHeaderName(err)
  }
}

impl From<header::InvalidHeaderValue> for Error {
  fn from(err: header::InvalidHeaderValue) -> Self {
    Error::InvalidHeaderValue(err)
  }
}

impl From<std::io::Error> for Error {
  fn from(err: std::io::Error) -> Self {
    Error::IO(err)
  }
}

impl From<wreq::Error> for Error {
  fn from(err: wreq::Error) -> Self {
    Error::Library(err)
  }
}

impl From<tokio::time::error::Elapsed> for Error {
  fn from(err: tokio::time::error::Elapsed) -> Self {
    Error::Timeout(err)
  }
}

impl From<ParseError> for Error {
  fn from(err: ParseError) -> Self {
    Error::Decode(err)
  }
}

impl From<http::Error> for Error {
  fn from(err: http::Error) -> Self {
    Error::Builder(err)
  }
}
