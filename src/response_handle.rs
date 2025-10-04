use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use http::Version;
use napi::bindgen_prelude::{Buffer, Result};
use napi_derive::napi;
use rnet_bindings_core::response::Response;
use wreq::header::{HeaderMap, HeaderValue};

use crate::error::to_napi_error;

/// HTTP response handle with automatic resource cleanup.
///
/// Response resources (connections, body streams) are automatically cleaned up when
/// the object is garbage collected, similar to undici and the Fetch API.
///
/// # Automatic Cleanup
///
/// You don't need to manually call `close()` - resources are automatically released when:
/// - The response body is consumed via `text()`, `json()`, or `bytes()`
/// - The JavaScript object is garbage collected
///
/// # Example
///
/// ```javascript
/// // Automatic cleanup - no close() needed
/// const response = await client.get('https://api.example.com/data');
/// const data = await response.json();
/// // Resources automatically cleaned up
///
/// // Optional explicit cleanup
/// const response = await client.get('https://api.example.com/data');
/// console.log(response.status);
/// response.close(); // Immediate cleanup (optional)
/// ```
#[napi]
pub struct ResponseHandle {
  inner: Arc<Response>,
  /// Track if the body has been consumed to avoid closing prematurely
  consumed: Arc<AtomicBool>,
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
      consumed: Arc::new(AtomicBool::new(false)),
    }
  }

  pub fn from_shared(inner: Arc<Response>) -> Self {
    Self {
      inner,
      consumed: Arc::new(AtomicBool::new(false)),
    }
  }

  pub fn as_shared(&self) -> Arc<Response> {
    Arc::clone(&self.inner)
  }

  fn mark_consumed(&self) {
    self.consumed.store(true, Ordering::Release);
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

  /// Reads the response body as text.
  /// The response is automatically cleaned up after consumption.
  #[napi]
  pub async fn text(&self) -> Result<String> {
    let result = self.inner.text().await.map_err(to_napi_error)?;
    self.mark_consumed();
    Ok(result)
  }

  /// Reads the response body as JSON.
  /// The response is automatically cleaned up after consumption.
  #[napi]
  pub async fn json(&self) -> Result<serde_json::Value> {
    let result = self.inner.json().await.map_err(to_napi_error)?;
    self.mark_consumed();
    Ok(result)
  }

  /// Reads the response body as raw bytes.
  /// The response is automatically cleaned up after consumption.
  #[napi]
  pub async fn bytes(&self) -> Result<Buffer> {
    let bytes = self.inner.bytes().await.map_err(to_napi_error)?;
    self.mark_consumed();
    Ok(bytes.to_vec().into())
  }

  /// Explicitly closes the response and releases resources immediately.
  ///
  /// **Note:** This method is optional. Response resources are automatically
  /// cleaned up when the object is garbage collected (similar to undici/fetch).
  /// Use this method only when you need immediate resource cleanup, such as in
  /// high-volume scenarios or long-running processes.
  ///
  /// # Example
  /// ```javascript
  /// // Automatic cleanup (recommended)
  /// const response = await client.get(url);
  /// const data = await response.json();
  /// // No close() needed - automatic cleanup on GC
  ///
  /// // Explicit cleanup (optional)
  /// const response = await client.get(url);
  /// console.log(response.status);
  /// response.close(); // Immediate cleanup
  /// ```
  #[napi]
  pub fn close(&self) {
    self.inner.close();
    self.mark_consumed();
  }
}

// Implement Drop trait for automatic cleanup when JavaScript object is garbage collected
// This provides the same behavior as undici/fetch where users don't need to manually close responses
impl Drop for ResponseHandle {
  fn drop(&mut self) {
    // Only close if the response body was never consumed
    // This prevents closing an already-consumed response and avoids double-free issues
    if !self.consumed.load(Ordering::Acquire) {
      self.inner.close();
    }
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
