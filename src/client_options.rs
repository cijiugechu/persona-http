use std::collections::HashMap;

use napi::bindgen_prelude::{Either, Result as NapiResult};
use napi_derive::napi;
use nitai_bindings_core::client::{ClientBuilder, TlsVerification};
use wreq::tls;

use crate::emulation::{parse_optional_emulation, EmulationOptions};
use crate::request_options::{
  convert_header_map, duration_from_millis, napi_invalid, parse_ip, parse_proxy, ProxyConfig,
};

#[napi(object)]
pub struct ClientInit {
  pub emulation: Option<Either<String, EmulationOptions>>,
  pub user_agent: Option<String>,
  pub headers: Option<HashMap<String, Either<String, Vec<String>>>>,
  pub referer: Option<bool>,
  pub history: Option<bool>,
  pub allow_redirects: Option<bool>,
  pub max_redirects: Option<u32>,
  pub cookie_store: Option<bool>,
  pub timeout: Option<u32>,
  pub connect_timeout: Option<u32>,
  pub read_timeout: Option<u32>,
  pub tcp_keepalive: Option<u32>,
  pub tcp_keepalive_interval: Option<u32>,
  pub tcp_keepalive_retries: Option<u32>,
  pub tcp_user_timeout: Option<u32>,
  pub tcp_nodelay: Option<bool>,
  pub tcp_reuse_address: Option<bool>,
  pub pool_idle_timeout: Option<u32>,
  pub pool_max_idle_per_host: Option<u32>,
  pub pool_max_size: Option<u32>,
  pub http1_only: Option<bool>,
  pub http2_only: Option<bool>,
  pub https_only: Option<bool>,
  pub min_tls_version: Option<String>,
  pub max_tls_version: Option<String>,
  pub tls_info: Option<bool>,
  pub verify: Option<Either<bool, String>>,
  pub verify_hostname: Option<bool>,
  pub no_proxy: Option<bool>,
  pub proxies: Option<Vec<ProxyConfig>>,
  pub local_address: Option<String>,
  pub interface: Option<String>,
  pub gzip: Option<bool>,
  pub brotli: Option<bool>,
  pub deflate: Option<bool>,
  pub zstd: Option<bool>,
}

impl ClientInit {
  pub fn build(self) -> NapiResult<ClientBuilder> {
    let mut builder = ClientBuilder::default();

    if let Some(emulation) = parse_optional_emulation(self.emulation)? {
      builder.emulation = Some(emulation);
    }

    if let Some(user_agent) = self.user_agent {
      builder.user_agent = Some(user_agent);
    }

    if let Some(headers) = self.headers {
      builder.headers = Some(convert_header_map(headers)?);
    }

    builder.referer = self.referer;
    builder.history = self.history;
    builder.allow_redirects = self.allow_redirects;
    builder.max_redirects = self.max_redirects.map(|v| v as usize);
    builder.cookie_store = self.cookie_store;

    if let Some(timeout) = self.timeout {
      builder.timeout = Some(duration_from_millis(timeout));
    }

    if let Some(connect_timeout) = self.connect_timeout {
      builder.connect_timeout = Some(duration_from_millis(connect_timeout));
    }

    if let Some(read_timeout) = self.read_timeout {
      builder.read_timeout = Some(duration_from_millis(read_timeout));
    }

    if let Some(keepalive) = self.tcp_keepalive {
      builder.tcp_keepalive = Some(duration_from_millis(keepalive));
    }

    if let Some(keepalive_interval) = self.tcp_keepalive_interval {
      builder.tcp_keepalive_interval = Some(duration_from_millis(keepalive_interval));
    }

    builder.tcp_keepalive_retries = self.tcp_keepalive_retries;
    builder.tcp_user_timeout = self.tcp_user_timeout.map(duration_from_millis);
    builder.tcp_nodelay = self.tcp_nodelay;
    builder.tcp_reuse_address = self.tcp_reuse_address;
    builder.pool_idle_timeout = self.pool_idle_timeout.map(duration_from_millis);
    builder.pool_max_idle_per_host = self.pool_max_idle_per_host.map(|v| v as usize);
    builder.pool_max_size = self.pool_max_size;
    builder.http1_only = self.http1_only;
    builder.http2_only = self.http2_only;
    builder.https_only = self.https_only;
    builder.tls_info = self.tls_info;
    builder.verify_hostname = self.verify_hostname;
    builder.no_proxy = self.no_proxy;
    builder.gzip = self.gzip;
    builder.brotli = self.brotli;
    builder.deflate = self.deflate;
    builder.zstd = self.zstd;

    if let Some(min_tls) = self.min_tls_version {
      builder.min_tls_version = Some(parse_tls_version(&min_tls)?);
    }

    if let Some(max_tls) = self.max_tls_version {
      builder.max_tls_version = Some(parse_tls_version(&max_tls)?);
    }

    if let Some(verify) = self.verify {
      builder.verify = Some(parse_verify(verify)?);
    }

    if let Some(proxies) = self.proxies {
      builder.proxies = Some(
        proxies
          .into_iter()
          .map(parse_proxy)
          .collect::<NapiResult<Vec<_>>>()?,
      );
    }

    if let Some(local_address) = self.local_address {
      builder.local_address = Some(parse_ip(local_address)?);
    }

    if let Some(interface) = self.interface {
      builder.interface = Some(interface);
    }

    Ok(builder)
  }
}

fn parse_tls_version(value: &str) -> NapiResult<tls::TlsVersion> {
  match value.to_ascii_uppercase().as_str() {
    "TLS1.0" | "TLS1" | "1.0" => Ok(tls::TlsVersion::TLS_1_0),
    "TLS1.1" | "1.1" => Ok(tls::TlsVersion::TLS_1_1),
    "TLS1.2" | "1.2" => Ok(tls::TlsVersion::TLS_1_2),
    "TLS1.3" | "1.3" => Ok(tls::TlsVersion::TLS_1_3),
    other => Err(napi_invalid(format!("unsupported TLS version: {other}"))),
  }
}

fn parse_verify(option: Either<bool, String>) -> NapiResult<TlsVerification> {
  match option {
    Either::A(enabled) => Ok(TlsVerification::Verification(enabled)),
    Either::B(path) => Ok(TlsVerification::CertificatePath(path.into())),
  }
}
