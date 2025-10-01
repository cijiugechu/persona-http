use std::collections::HashMap;
use std::sync::Arc;

use http::Version;
use napi::bindgen_prelude::{Buffer, Result};
use napi_derive::napi;
use rnet_bindings_core::response::Response;
use wreq::header::{HeaderMap, HeaderValue};

use crate::error::to_napi_error;

#[napi]
pub struct ResponseHandle {
  inner: Arc<Response>,
}

#[napi(object)]
pub struct RedirectHistoryEntry {
  pub status: u16,
  pub uri: String,
  pub previous: String,
}

impl ResponseHandle {
  pub fn new(response: Response) -> Self {
    Self {
      inner: Arc::new(response),
    }
  }

  pub fn from_shared(inner: Arc<Response>) -> Self {
    Self { inner }
  }

  pub fn as_shared(&self) -> Arc<Response> {
    Arc::clone(&self.inner)
  }
}

#[napi]
impl ResponseHandle {
  #[napi(getter)]
  pub fn status(&self) -> u16 {
    self.inner.status.as_u16()
  }

  #[napi(getter)]
  pub fn ok(&self) -> bool {
    self.inner.status.is_success()
  }

  #[napi(getter)]
  pub fn status_text(&self) -> String {
    self
      .inner
      .status
      .canonical_reason()
      .unwrap_or_default()
      .to_string()
  }

  #[napi(getter)]
  pub fn url(&self) -> String {
    self.inner.uri.to_string()
  }

  #[napi(getter)]
  pub fn version(&self) -> String {
    format_version(self.inner.version)
  }

  /// Returns the response `Content-Length` in bytes, or `-1` when not provided by the server.
  #[napi(getter)]
  pub fn content_length(&self) -> i64 {
    match self.inner.content_length {
      Some(len) if len > i64::MAX as u64 => i64::MAX,
      Some(len) => len as i64,
      None => -1,
    }
  }

  #[napi(getter)]
  pub fn headers(&self) -> HashMap<String, Vec<String>> {
    flatten_headers(&self.inner.headers)
  }

  #[napi(getter)]
  pub fn local_addr(&self) -> Option<String> {
    self.inner.local_addr.map(|addr| addr.to_string())
  }

  #[napi(getter)]
  pub fn remote_addr(&self) -> Option<String> {
    self.inner.remote_addr.map(|addr| addr.to_string())
  }

  #[napi]
  pub fn history(&self) -> Vec<RedirectHistoryEntry> {
    self
      .inner
      .history()
      .into_iter()
      .map(|history| RedirectHistoryEntry {
        status: history.status().as_u16(),
        uri: history.uri().to_string(),
        previous: history.previous().to_string(),
      })
      .collect()
  }

  #[napi]
  pub async fn text(&self) -> Result<String> {
    self.inner.text().await.map_err(to_napi_error)
  }

  #[napi]
  pub async fn json(&self) -> Result<serde_json::Value> {
    self.inner.json().await.map_err(to_napi_error)
  }

  #[napi]
  pub async fn bytes(&self) -> Result<Buffer> {
    let bytes = self.inner.bytes().await.map_err(to_napi_error)?;
    Ok(bytes.to_vec().into())
  }

  #[napi]
  pub fn close(&self) {
    self.inner.close();
  }
}

fn format_version(version: Version) -> String {
  match version {
    Version::HTTP_09 => "HTTP/0.9".into(),
    Version::HTTP_10 => "HTTP/1.0".into(),
    Version::HTTP_11 => "HTTP/1.1".into(),
    Version::HTTP_2 => "HTTP/2".into(),
    Version::HTTP_3 => "HTTP/3".into(),
    other => format!("{other:?}"),
  }
}

fn flatten_headers(headers: &HeaderMap) -> HashMap<String, Vec<String>> {
  let mut map: HashMap<String, Vec<String>> = HashMap::new();
  for (name, value) in headers.iter() {
    map
      .entry(name.to_string())
      .or_default()
      .push(header_value_to_string(value));
  }
  map
}

fn header_value_to_string(value: &HeaderValue) -> String {
  match value.to_str() {
    Ok(s) => s.to_string(),
    Err(_) => String::from_utf8_lossy(value.as_bytes()).into_owned(),
  }
}
