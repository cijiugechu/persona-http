mod dns;

use std::{
    fs,
    net::IpAddr,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use wreq::{self, Proxy};
use wreq::redirect::Policy;
use wreq_util::EmulationOption;

use crate::{
    Error,
    Request,
    Response,
    WebSocket,
    WebSocketRequest,
};

pub use dns::HickoryDnsResolver;

/// Wrapper around the underlying wreq client.
#[derive(Clone)]
pub struct Client {
    inner: wreq::Client,
}

impl Client {
    pub fn new(inner: wreq::Client) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &wreq::Client {
        &self.inner
    }

    pub fn into_inner(self) -> wreq::Client {
        self.inner
    }
}

impl From<wreq::Client> for Client {
    fn from(inner: wreq::Client) -> Self {
        Self::new(inner)
    }
}

/// TLS verification configuration.
#[derive(Clone)]
pub enum TlsVerification {
    Verification(bool),
    CertificatePath(PathBuf),
    CertificateStore(wreq::tls::CertStore),
}

/// Builder for [`Client`].
#[derive(Default, Clone)]
pub struct ClientBuilder {
    pub emulation: Option<EmulationOption>,
    pub user_agent: Option<String>,
    pub headers: Option<wreq::header::HeaderMap>,
    pub orig_headers: Option<wreq::header::OrigHeaderMap>,
    pub referer: Option<bool>,
    pub history: Option<bool>,
    pub allow_redirects: Option<bool>,
    pub max_redirects: Option<usize>,
    pub cookie_store: Option<bool>,
    pub cookie_provider: Option<Arc<wreq::cookie::Jar>>,
    pub timeout: Option<Duration>,
    pub connect_timeout: Option<Duration>,
    pub read_timeout: Option<Duration>,
    pub tcp_keepalive: Option<Duration>,
    pub tcp_keepalive_interval: Option<Duration>,
    pub tcp_keepalive_retries: Option<u32>,
    pub tcp_user_timeout: Option<Duration>,
    pub tcp_nodelay: Option<bool>,
    pub tcp_reuse_address: Option<bool>,
    pub pool_idle_timeout: Option<Duration>,
    pub pool_max_idle_per_host: Option<usize>,
    pub pool_max_size: Option<u32>,
    pub http1_only: Option<bool>,
    pub http2_only: Option<bool>,
    pub https_only: Option<bool>,
    pub http1_options: Option<wreq::http1::Http1Options>,
    pub http2_options: Option<wreq::http2::Http2Options>,
    pub verify: Option<TlsVerification>,
    pub verify_hostname: Option<bool>,
    pub identity: Option<wreq::tls::Identity>,
    pub keylog: Option<wreq::tls::KeyLog>,
    pub tls_info: Option<bool>,
    pub min_tls_version: Option<wreq::tls::TlsVersion>,
    pub max_tls_version: Option<wreq::tls::TlsVersion>,
    pub tls_options: Option<wreq::tls::TlsOptions>,
    pub no_proxy: Option<bool>,
    pub proxies: Option<Vec<Proxy>>,
    pub local_address: Option<IpAddr>,
    pub interface: Option<String>,
    pub gzip: Option<bool>,
    pub brotli: Option<bool>,
    pub deflate: Option<bool>,
    pub zstd: Option<bool>,
}

impl ClientBuilder {
    pub fn build(mut self) -> Result<Client, Error> {
        let mut builder = wreq::Client::builder();

        if let Some(emulation) = self.emulation.take() {
            builder = builder.emulation(emulation);
        }

        if let Some(user_agent) = self.user_agent.take() {
            builder = builder.user_agent(&user_agent);
        }

        if let Some(headers) = self.headers.take() {
            builder = builder.default_headers(headers);
        }

        if let Some(orig_headers) = self.orig_headers.take() {
            builder = builder.orig_headers(orig_headers);
        }

        if let Some(referer) = self.referer.take() {
            builder = builder.referer(referer);
        }

        if let Some(history) = self.history.take() {
            builder = builder.history(history);
        }

        match (self.allow_redirects.take(), self.max_redirects.take()) {
            (Some(false), _) => {
                builder = builder.redirect(Policy::none());
            }
            (Some(true), max_redirects) => {
                let policy = max_redirects
                    .map(Policy::limited)
                    .unwrap_or_else(Policy::default);
                builder = builder.redirect(policy);
            }
            (None, _) => {}
        }

        if let Some(cookie_provider) = self.cookie_provider.take() {
            builder = builder.cookie_provider(cookie_provider);
        } else if let Some(cookie_store) = self.cookie_store.take() {
            builder = builder.cookie_store(cookie_store);
        }

        if let Some(timeout) = self.timeout.take() {
            builder = builder.timeout(timeout);
        }

        if let Some(connect_timeout) = self.connect_timeout.take() {
            builder = builder.connect_timeout(connect_timeout);
        }

        if let Some(read_timeout) = self.read_timeout.take() {
            builder = builder.read_timeout(read_timeout);
        }

        if let Some(tcp_keepalive) = self.tcp_keepalive.take() {
            builder = builder.tcp_keepalive(tcp_keepalive);
        }

        if let Some(tcp_keepalive_interval) = self.tcp_keepalive_interval.take() {
            builder = builder.tcp_keepalive_interval(tcp_keepalive_interval);
        }

        if let Some(tcp_keepalive_retries) = self.tcp_keepalive_retries.take() {
            builder = builder.tcp_keepalive_retries(tcp_keepalive_retries);
        }

        #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
        if let Some(tcp_user_timeout) = self.tcp_user_timeout.take() {
            builder = builder.tcp_user_timeout(tcp_user_timeout);
        }

        if let Some(tcp_nodelay) = self.tcp_nodelay.take() {
            builder = builder.tcp_nodelay(tcp_nodelay);
        }

        if let Some(tcp_reuse_address) = self.tcp_reuse_address.take() {
            builder = builder.tcp_reuse_address(tcp_reuse_address);
        }

        if let Some(pool_idle_timeout) = self.pool_idle_timeout.take() {
            builder = builder.pool_idle_timeout(pool_idle_timeout);
        }

        if let Some(pool_max_idle_per_host) = self.pool_max_idle_per_host.take() {
            builder = builder.pool_max_idle_per_host(pool_max_idle_per_host);
        }

        if let Some(pool_max_size) = self.pool_max_size.take() {
            builder = builder.pool_max_size(pool_max_size);
        }

        if self.http1_only.unwrap_or(false) {
            builder = builder.http1_only();
        }

        if self.http2_only.unwrap_or(false) {
            builder = builder.http2_only();
        }

        if let Some(https_only) = self.https_only.take() {
            builder = builder.https_only(https_only);
        }

        if let Some(http1_options) = self.http1_options.take() {
            builder = builder.http1_options(http1_options);
        }

        if let Some(http2_options) = self.http2_options.take() {
            builder = builder.http2_options(http2_options);
        }

        if let Some(min_tls_version) = self.min_tls_version.take() {
            builder = builder.min_tls_version(min_tls_version);
        }

        if let Some(max_tls_version) = self.max_tls_version.take() {
            builder = builder.max_tls_version(max_tls_version);
        }

        if let Some(tls_info) = self.tls_info.take() {
            builder = builder.tls_info(tls_info);
        }

        if let Some(verify) = self.verify.take() {
            builder = match verify {
                TlsVerification::Verification(verify) => builder.cert_verification(verify),
                TlsVerification::CertificatePath(path) => {
                    let pem_data = fs::read(path)?;
                    let store = wreq::tls::CertStore::from_pem_stack(&pem_data)
                        .map_err(Error::Library)?;
                    builder.cert_store(store)
                }
                TlsVerification::CertificateStore(store) => builder.cert_store(store),
            };
        }

        if let Some(verify_hostname) = self.verify_hostname.take() {
            builder = builder.verify_hostname(verify_hostname);
        }

        if let Some(identity) = self.identity.take() {
            builder = builder.identity(identity);
        }

        if let Some(keylog) = self.keylog.take() {
            builder = builder.keylog(keylog);
        }

        if let Some(tls_options) = self.tls_options.take() {
            builder = builder.tls_options(tls_options);
        }

        if let Some(mut proxies) = self.proxies.take() {
            for proxy in proxies.drain(..) {
                builder = builder.proxy(proxy);
            }
        }

        if self.no_proxy.unwrap_or(false) {
            builder = builder.no_proxy();
        }

        if let Some(local_address) = self.local_address.take() {
            builder = builder.local_address(local_address);
        }

        #[cfg(any(
            target_os = "android",
            target_os = "fuchsia",
            target_os = "illumos",
            target_os = "ios",
            target_os = "linux",
            target_os = "macos",
            target_os = "solaris",
            target_os = "tvos",
            target_os = "visionos",
            target_os = "watchos",
        ))]
        if let Some(interface) = self.interface.take() {
            builder = builder.interface(interface);
        }

        if let Some(gzip) = self.gzip.take() {
            builder = builder.gzip(gzip);
        }

        if let Some(brotli) = self.brotli.take() {
            builder = builder.brotli(brotli);
        }

        if let Some(deflate) = self.deflate.take() {
            builder = builder.deflate(deflate);
        }

        if let Some(zstd) = self.zstd.take() {
            builder = builder.zstd(zstd);
        }

        builder
            .dns_resolver(HickoryDnsResolver::new())
            .build()
            .map(Client::new)
            .map_err(Error::Library)
    }
}

/// Execute an HTTP request using either an existing client or the global request builder.
pub async fn execute_request(
    client: Option<Client>,
    method: wreq::Method,
    url: &str,
    mut params: Request,
) -> Result<Response, Error> {
    let mut builder = match client {
        Some(client) => client.into_inner().request(method, url),
        None => wreq::request(method, url),
    };

    if let Some(emulation) = params.emulation.take() {
        builder = builder.emulation(emulation);
    }

    if let Some(version) = params.version.take() {
        builder = builder.version(version);
    }

    if let Some(timeout) = params.timeout.take() {
        builder = builder.timeout(timeout);
    }

    if let Some(read_timeout) = params.read_timeout.take() {
        builder = builder.read_timeout(read_timeout);
    }

    if let Some(proxy) = params.proxy.take() {
        builder = builder.proxy(proxy);
    }

    if let Some(local_address) = params.local_address.take() {
        builder = builder.local_address(local_address);
    }

    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "illumos",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "solaris",
        target_os = "tvos",
        target_os = "visionos",
        target_os = "watchos",
    ))]
    if let Some(interface) = params.interface.take() {
        builder = builder.interface(interface);
    }

    if let Some(headers) = params.headers.take() {
        builder = builder.headers(headers);
    }

    if let Some(orig_headers) = params.orig_headers.take() {
        builder = builder.orig_headers(orig_headers);
    }

    if let Some(default_headers) = params.default_headers.take() {
        builder = builder.default_headers(default_headers);
    }

    if let Some(auth) = params.auth.take() {
        builder = builder.auth(auth);
    }

    if let Some(bearer) = params.bearer_auth.take() {
        builder = builder.bearer_auth(bearer);
    }

    if let Some((username, password)) = params.basic_auth.take() {
        builder = builder.basic_auth(username, password);
    }

    if let Some(cookies) = params.cookies.take() {
        for cookie in cookies {
            builder = builder.header_append(wreq::header::COOKIE, cookie);
        }
    }

    match (params.allow_redirects.take(), params.max_redirects.take()) {
        (Some(false), _) => {
            builder = builder.redirect(Policy::none());
        }
        (Some(true), max_redirects) => {
            let policy = max_redirects
                .map(Policy::limited)
                .unwrap_or_else(Policy::default);
            builder = builder.redirect(policy);
        }
        (None, _) => {}
    }

    if let Some(query) = params.query.take() {
        builder = builder.query(&query);
    }

    if let Some(form) = params.form.take() {
        builder = builder.form(&form);
    }

    if let Some(json) = params.json.take() {
        builder = builder.json(&json);
    }

    if let Some(body) = params.body.take() {
        builder = builder.body(body);
    }

    if let Some(multipart) = params.multipart.take() {
        builder = builder.multipart(multipart);
    }

    if let Some(gzip) = params.gzip.take() {
        builder = builder.gzip(gzip);
    }

    if let Some(brotli) = params.brotli.take() {
        builder = builder.brotli(brotli);
    }

    if let Some(deflate) = params.deflate.take() {
        builder = builder.deflate(deflate);
    }

    if let Some(zstd) = params.zstd.take() {
        builder = builder.zstd(zstd);
    }

    builder
        .send()
        .await
        .map(Response::new)
        .map_err(Error::Library)
}

/// Execute a WebSocket request using either an existing client or the global builder.
pub async fn execute_websocket_request(
    client: Option<Client>,
    url: &str,
    mut params: WebSocketRequest,
) -> Result<WebSocket, Error> {
    let mut builder = match client {
        Some(client) => client.into_inner().websocket(url),
        None => wreq::websocket(url),
    };

    if let Some(protocols) = params.protocols.take() {
        builder = builder.protocols(protocols);
    }

    if let Some(read_buffer_size) = params.read_buffer_size.take() {
        builder = builder.read_buffer_size(read_buffer_size);
    }

    if let Some(write_buffer_size) = params.write_buffer_size.take() {
        builder = builder.write_buffer_size(write_buffer_size);
    }

    if let Some(max_write_buffer_size) = params.max_write_buffer_size.take() {
        builder = builder.max_write_buffer_size(max_write_buffer_size);
    }

    if let Some(max_frame_size) = params.max_frame_size.take() {
        builder = builder.max_frame_size(max_frame_size);
    }

    if let Some(max_message_size) = params.max_message_size.take() {
        builder = builder.max_message_size(max_message_size);
    }

    if let Some(accept_unmasked_frames) = params.accept_unmasked_frames.take() {
        builder = builder.accept_unmasked_frames(accept_unmasked_frames);
    }

    if params.force_http2.unwrap_or(false) {
        builder = builder.force_http2();
    }

    if let Some(proxy) = params.proxy.take() {
        builder = builder.proxy(proxy);
    }

    if let Some(local_address) = params.local_address.take() {
        builder = builder.local_address(local_address);
    }

    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "illumos",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "solaris",
        target_os = "tvos",
        target_os = "visionos",
        target_os = "watchos",
    ))]
    if let Some(interface) = params.interface.take() {
        builder = builder.interface(interface);
    }

    if let Some(headers) = params.headers.take() {
        builder = builder.headers(headers);
    }

    if let Some(orig_headers) = params.orig_headers.take() {
        builder = builder.orig_headers(orig_headers);
    }

    if let Some(default_headers) = params.default_headers.take() {
        builder = builder.default_headers(default_headers);
    }

    if let Some(auth) = params.auth.take() {
        builder = builder.auth(auth);
    }

    if let Some(bearer) = params.bearer_auth.take() {
        builder = builder.bearer_auth(bearer);
    }

    if let Some((username, password)) = params.basic_auth.take() {
        builder = builder.basic_auth(username, password);
    }

    if let Some(cookies) = params.cookies.take() {
        for cookie in cookies {
            builder = builder.header_append(wreq::header::COOKIE, cookie);
        }
    }

    if let Some(query) = params.query.take() {
        builder = builder.query(&query);
    }

    let response = builder.send().await.map_err(Error::Library)?;
    WebSocket::new(response).await
}
