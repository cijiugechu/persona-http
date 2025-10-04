#![deny(clippy::all)]

mod client_options;
mod emulation;
mod error;
mod request_options;
mod response_handle;

pub use client_options::ClientInit;
pub use request_options::{BasicAuth, ProxyConfig, RequestInit, WebSocketInit};
pub use response_handle::{RedirectHistoryEntry, ResponseHandle};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use nitai_bindings_core::{
  client::{Client as CoreClient, ClientBuilder},
  execute_request,
};
use wreq::Method;

use crate::error::to_napi_error;
use crate::request_options::{parse_method, ParsedRequest};

#[napi]
pub struct Client {
  inner: CoreClient,
}

#[napi]
impl Client {
  #[napi(constructor)]
  pub fn new(init: Option<ClientInit>) -> Result<Self> {
    let builder = if let Some(init) = init {
      init.build()?
    } else {
      ClientBuilder::default()
    };

    let client = builder.build().map_err(to_napi_error)?;
    Ok(Self { inner: client })
  }

  #[napi]
  pub async fn request(
    &self,
    method: String,
    url: String,
    init: Option<RequestInit>,
  ) -> Result<ResponseHandle> {
    let override_method = Some(parse_method(&method)?);
    perform_request(
      Some(self.inner.clone()),
      url,
      init,
      Method::GET,
      override_method,
    )
    .await
  }

  #[napi]
  pub async fn get(&self, url: String, init: Option<RequestInit>) -> Result<ResponseHandle> {
    perform_request(
      Some(self.inner.clone()),
      url,
      init,
      Method::GET,
      Some(Method::GET),
    )
    .await
  }

  #[napi]
  pub async fn post(&self, url: String, init: Option<RequestInit>) -> Result<ResponseHandle> {
    perform_request(
      Some(self.inner.clone()),
      url,
      init,
      Method::POST,
      Some(Method::POST),
    )
    .await
  }

  #[napi]
  pub async fn put(&self, url: String, init: Option<RequestInit>) -> Result<ResponseHandle> {
    perform_request(
      Some(self.inner.clone()),
      url,
      init,
      Method::PUT,
      Some(Method::PUT),
    )
    .await
  }

  #[napi]
  pub async fn patch(&self, url: String, init: Option<RequestInit>) -> Result<ResponseHandle> {
    perform_request(
      Some(self.inner.clone()),
      url,
      init,
      Method::PATCH,
      Some(Method::PATCH),
    )
    .await
  }

  #[napi]
  pub async fn delete(&self, url: String, init: Option<RequestInit>) -> Result<ResponseHandle> {
    perform_request(
      Some(self.inner.clone()),
      url,
      init,
      Method::DELETE,
      Some(Method::DELETE),
    )
    .await
  }

  #[napi]
  pub async fn head(&self, url: String, init: Option<RequestInit>) -> Result<ResponseHandle> {
    perform_request(
      Some(self.inner.clone()),
      url,
      init,
      Method::HEAD,
      Some(Method::HEAD),
    )
    .await
  }

  #[napi]
  pub async fn options(&self, url: String, init: Option<RequestInit>) -> Result<ResponseHandle> {
    perform_request(
      Some(self.inner.clone()),
      url,
      init,
      Method::OPTIONS,
      Some(Method::OPTIONS),
    )
    .await
  }
}

#[napi]
pub async fn request(url: String, init: Option<RequestInit>) -> Result<ResponseHandle> {
  perform_request(None, url, init, Method::GET, None).await
}

#[napi]
pub async fn get(url: String, init: Option<RequestInit>) -> Result<ResponseHandle> {
  perform_request(None, url, init, Method::GET, Some(Method::GET)).await
}

#[napi]
pub async fn post(url: String, init: Option<RequestInit>) -> Result<ResponseHandle> {
  perform_request(None, url, init, Method::POST, Some(Method::POST)).await
}

#[napi]
pub async fn put(url: String, init: Option<RequestInit>) -> Result<ResponseHandle> {
  perform_request(None, url, init, Method::PUT, Some(Method::PUT)).await
}

#[napi]
pub async fn patch(url: String, init: Option<RequestInit>) -> Result<ResponseHandle> {
  perform_request(None, url, init, Method::PATCH, Some(Method::PATCH)).await
}

#[napi(js_name = "delete_")]
/// use this instead of delete because delete is a reserved keyword in JavaScript
pub async fn delete(url: String, init: Option<RequestInit>) -> Result<ResponseHandle> {
  perform_request(None, url, init, Method::DELETE, Some(Method::DELETE)).await
}

#[napi]
pub async fn head(url: String, init: Option<RequestInit>) -> Result<ResponseHandle> {
  perform_request(None, url, init, Method::HEAD, Some(Method::HEAD)).await
}

#[napi]
pub async fn options(url: String, init: Option<RequestInit>) -> Result<ResponseHandle> {
  perform_request(None, url, init, Method::OPTIONS, Some(Method::OPTIONS)).await
}

async fn perform_request(
  client: Option<CoreClient>,
  url: String,
  init: Option<RequestInit>,
  default_method: Method,
  override_method: Option<Method>,
) -> Result<ResponseHandle> {
  let ParsedRequest { method, request } = match init {
    Some(init) => init.parse()?,
    None => ParsedRequest::default(),
  };

  let method = override_method.or(method).unwrap_or(default_method);

  let response = execute_request(client, method, &url, request)
    .await
    .map_err(to_napi_error)?;

  Ok(ResponseHandle::new(response))
}
