use std::{net::IpAddr, time::Duration};

use wreq::{
  header::{HeaderMap, HeaderValue, OrigHeaderMap},
  multipart::Form,
  Proxy, Version,
};
use wreq_util::EmulationOption;

/// The parameters for an HTTP request.
#[derive(Default)]
#[non_exhaustive]
pub struct Request {
  pub emulation: Option<EmulationOption>,
  pub proxy: Option<Proxy>,
  pub local_address: Option<IpAddr>,
  pub interface: Option<String>,
  pub timeout: Option<Duration>,
  pub read_timeout: Option<Duration>,
  pub version: Option<Version>,
  pub headers: Option<HeaderMap>,
  pub orig_headers: Option<OrigHeaderMap>,
  pub default_headers: Option<bool>,
  pub cookies: Option<Vec<HeaderValue>>,
  pub allow_redirects: Option<bool>,
  pub max_redirects: Option<usize>,
  pub gzip: Option<bool>,
  pub brotli: Option<bool>,
  pub deflate: Option<bool>,
  pub zstd: Option<bool>,
  pub auth: Option<String>,
  pub bearer_auth: Option<String>,
  pub basic_auth: Option<(String, Option<String>)>,
  pub query: Option<Vec<(String, String)>>,
  pub form: Option<Vec<(String, String)>>,
  pub json: Option<serde_json::Value>,
  pub body: Option<wreq::Body>,
  pub multipart: Option<Form>,
}

impl Request {
  pub fn is_empty(&self) -> bool {
    let Request {
      emulation,
      proxy,
      local_address,
      interface,
      timeout,
      read_timeout,
      version,
      headers,
      orig_headers,
      default_headers,
      cookies,
      allow_redirects,
      max_redirects,
      gzip,
      brotli,
      deflate,
      zstd,
      auth,
      bearer_auth,
      basic_auth,
      query,
      form,
      json,
      body,
      multipart,
    } = self;

    emulation.is_none()
      && proxy.is_none()
      && local_address.is_none()
      && interface.is_none()
      && timeout.is_none()
      && read_timeout.is_none()
      && version.is_none()
      && headers.is_none()
      && orig_headers.is_none()
      && default_headers.is_none()
      && cookies.is_none()
      && allow_redirects.is_none()
      && max_redirects.is_none()
      && gzip.is_none()
      && brotli.is_none()
      && deflate.is_none()
      && zstd.is_none()
      && auth.is_none()
      && bearer_auth.is_none()
      && basic_auth.is_none()
      && query.is_none()
      && form.is_none()
      && json.is_none()
      && body.is_none()
      && multipart.is_none()
  }
}

/// The parameters for a WebSocket request.
#[derive(Default)]
#[non_exhaustive]
pub struct WebSocketRequest {
  pub emulation: Option<EmulationOption>,
  pub proxy: Option<Proxy>,
  pub local_address: Option<IpAddr>,
  pub interface: Option<String>,
  pub headers: Option<HeaderMap>,
  pub orig_headers: Option<OrigHeaderMap>,
  pub default_headers: Option<bool>,
  pub cookies: Option<Vec<HeaderValue>>,
  pub protocols: Option<Vec<String>>,
  pub force_http2: Option<bool>,
  pub auth: Option<String>,
  pub bearer_auth: Option<String>,
  pub basic_auth: Option<(String, Option<String>)>,
  pub query: Option<Vec<(String, String)>>,
  pub read_buffer_size: Option<usize>,
  pub write_buffer_size: Option<usize>,
  pub max_write_buffer_size: Option<usize>,
  pub max_frame_size: Option<usize>,
  pub max_message_size: Option<usize>,
  pub accept_unmasked_frames: Option<bool>,
}

impl WebSocketRequest {
  pub fn is_empty(&self) -> bool {
    let WebSocketRequest {
      emulation,
      proxy,
      local_address,
      interface,
      headers,
      orig_headers,
      default_headers,
      cookies,
      protocols,
      force_http2,
      auth,
      bearer_auth,
      basic_auth,
      query,
      read_buffer_size,
      write_buffer_size,
      max_write_buffer_size,
      max_frame_size,
      max_message_size,
      accept_unmasked_frames,
    } = self;

    emulation.is_none()
      && proxy.is_none()
      && local_address.is_none()
      && interface.is_none()
      && headers.is_none()
      && orig_headers.is_none()
      && default_headers.is_none()
      && cookies.is_none()
      && protocols.is_none()
      && force_http2.is_none()
      && auth.is_none()
      && bearer_auth.is_none()
      && basic_auth.is_none()
      && query.is_none()
      && read_buffer_size.is_none()
      && write_buffer_size.is_none()
      && max_write_buffer_size.is_none()
      && max_frame_size.is_none()
      && max_message_size.is_none()
      && accept_unmasked_frames.is_none()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn request_is_empty_by_default() {
    let request = Request::default();
    assert!(request.is_empty());
  }

  #[test]
  fn request_reports_non_empty_when_field_set() {
    let mut request = Request::default();
    request.timeout = Some(Duration::from_secs(1));
    assert!(!request.is_empty());
  }

  #[test]
  fn websocket_request_is_empty_by_default() {
    let request = WebSocketRequest::default();
    assert!(request.is_empty());
  }

  #[test]
  fn websocket_request_reports_non_empty_when_field_set() {
    let mut request = WebSocketRequest::default();
    request.force_http2 = Some(true);
    assert!(!request.is_empty());
  }
}
