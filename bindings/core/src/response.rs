use std::{
    net::SocketAddr,
    sync::Arc,
};

use arc_swap::ArcSwapOption;
use bytes::Bytes;
use futures_util::TryFutureExt;
use http::{Extensions, StatusCode, Uri, Version, response::Response as HttpResponse};
use http_body_util::BodyExt;
use wreq::{self, Extension};

use crate::error::Error;

/// Represents the state of the HTTP response body.
#[derive(Debug)]
pub enum ResponseBody {
    /// The body can be streamed once (not yet buffered).
    Streamable(wreq::Body),
    /// The body has been fully read into memory and can be reused.
    Reusable(Bytes),
}

/// A binding-agnostic HTTP response wrapper.
#[derive(Debug)]
pub struct Response {
    pub version: Version,
    pub status: StatusCode,
    pub content_length: Option<u64>,
    pub headers: wreq::header::HeaderMap,
    pub local_addr: Option<SocketAddr>,
    pub remote_addr: Option<SocketAddr>,
    pub uri: Uri,
    pub extensions: Extensions,
    body: ArcSwapOption<ResponseBody>,
}

impl Response {
    /// Construct a new [`Response`] from a wreq response.
    pub fn new(response: wreq::Response) -> Self {
        let uri = response.uri().clone();
        let content_length = response.content_length();
        let local_addr = response.local_addr();
        let remote_addr = response.remote_addr();
        let response = HttpResponse::from(response);
        let (parts, body) = response.into_parts();

        Response {
            uri,
            local_addr,
            remote_addr,
            content_length,
            extensions: parts.extensions,
            version: parts.version,
            status: parts.status,
            headers: parts.headers,
            body: ArcSwapOption::from_pointee(ResponseBody::Streamable(body)),
        }
    }

    /// Attempt to reuse the response body, yielding a fresh [`wreq::Response`].
    async fn reuse_response(&self, stream: bool) -> Result<wreq::Response, Error> {
        let build_response = |body: wreq::Body| -> wreq::Response {
            let mut response = HttpResponse::new(body);
            *response.version_mut() = self.version;
            *response.status_mut() = self.status;
            *response.headers_mut() = self.headers.clone();
            *response.extensions_mut() = self.extensions.clone();
            wreq::Response::from(response)
        };

        if let Some(arc) = self.body.swap(None) {
            match Arc::try_unwrap(arc) {
                Ok(ResponseBody::Streamable(body)) => {
                    if stream {
                        Ok(build_response(body))
                    } else {
                        let bytes = BodyExt::collect(body)
                            .map_ok(|buf| buf.to_bytes())
                            .map_err(Error::Library)
                            .await?;
                        self.body
                            .store(Some(Arc::new(ResponseBody::Reusable(bytes.clone()))));
                        Ok(build_response(wreq::Body::from(bytes)))
                    }
                }
                Ok(ResponseBody::Reusable(bytes)) => {
                    let cloned = bytes.clone();
                    self.body
                        .store(Some(Arc::new(ResponseBody::Reusable(bytes))));
                    Ok(build_response(wreq::Body::from(cloned)))
                }
                Err(arc) => {
                    self.body.store(Some(arc));
                    Err(Error::Memory)
                }
            }
        } else {
            Err(Error::Memory)
        }
    }

    /// Obtain a reusable response for operations that fully consume the body.
    pub async fn response(&self) -> Result<wreq::Response, Error> {
        self.reuse_response(false).await
    }

    /// Obtain a streaming response for operations that stream the body once.
    pub async fn response_for_stream(&self) -> Result<wreq::Response, Error> {
        self.reuse_response(true).await
    }

    /// Retrieve the text body.
    pub async fn text(&self) -> Result<String, Error> {
        self.reuse_response(false)
            .await?
            .text()
            .await
            .map_err(Error::Library)
    }

    /// Retrieve the text body with a specific charset.
    pub async fn text_with_charset(&self, encoding: &str) -> Result<String, Error> {
        self.reuse_response(false)
            .await?
            .text_with_charset(encoding)
            .await
            .map_err(Error::Library)
    }

    /// Retrieve the JSON body as a `serde_json::Value`.
    pub async fn json(&self) -> Result<serde_json::Value, Error> {
        self.reuse_response(false)
            .await?
            .json::<serde_json::Value>()
            .await
            .map_err(Error::Library)
    }

    /// Retrieve the raw bytes body.
    pub async fn bytes(&self) -> Result<Bytes, Error> {
        self.reuse_response(false)
            .await?
            .bytes()
            .await
            .map_err(Error::Library)
    }

    /// Close the response and drop any cached body state.
    pub fn close(&self) {
        self.body.swap(None);
    }

    /// Access the redirect history extension.
    pub fn history(&self) -> Vec<wreq::redirect::History> {
        self.extensions
            .get::<Extension<Vec<wreq::redirect::History>>>()
            .map(|Extension(history)| history.clone())
            .unwrap_or_default()
    }

    /// Access the TLS peer certificate, if available.
    pub fn peer_certificate(&self) -> Option<Bytes> {
        self.extensions
            .get::<Extension<wreq::tls::TlsInfo>>()
            .and_then(|Extension(info)| info.peer_certificate().map(Bytes::copy_from_slice))
    }
}

impl From<Response> for HttpResponse<wreq::Body> {
    fn from(response: Response) -> Self {
        let mut http_response = http::Response::new(wreq::Body::from(Bytes::new()));
        *http_response.version_mut() = response.version;
        *http_response.status_mut() = response.status;
        *http_response.headers_mut() = response.headers;
        *http_response.extensions_mut() = response.extensions;
        http_response
    }
}
