use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Duration;

use napi::bindgen_prelude::{Buffer, Either, Result as NapiResult};
use napi::{Error as NapiError, Status};
use napi_derive::napi;
use rnet_bindings_core::request::{Request, WebSocketRequest};
use wreq::header::{HeaderMap, HeaderName, HeaderValue};
use wreq::{self, Method, Proxy, Version};

use crate::emulation::{parse_optional_emulation, EmulationOptions};

#[napi(object)]
pub struct BasicAuth {
  pub username: String,
  pub password: Option<String>,
}

#[napi(object)]
pub struct ProxyConfig {
  pub uri: String,
  pub username: Option<String>,
  pub password: Option<String>,
}

#[napi(object)]
pub struct RequestInit {
  pub method: Option<String>,
  pub headers: Option<HashMap<String, Either<String, Vec<String>>>>,
  pub default_headers: Option<bool>,
  pub cookies: Option<Vec<String>>,
  pub emulation: Option<Either<String, EmulationOptions>>,
  pub allow_redirects: Option<bool>,
  pub max_redirects: Option<u32>,
  pub gzip: Option<bool>,
  pub brotli: Option<bool>,
  pub deflate: Option<bool>,
  pub zstd: Option<bool>,
  pub auth: Option<String>,
  pub bearer_auth: Option<String>,
  pub basic_auth: Option<BasicAuth>,
  pub query: Option<HashMap<String, Either<String, Vec<String>>>>,
  pub form: Option<HashMap<String, Either<String, Vec<String>>>>,
  pub json: Option<serde_json::Value>,
  pub body: Option<Either<String, Buffer>>,
  pub timeout: Option<u32>,
  pub read_timeout: Option<u32>,
  pub version: Option<String>,
  pub proxy: Option<ProxyConfig>,
  pub local_address: Option<String>,
  pub interface: Option<String>,
}

#[napi(object)]
pub struct WebSocketInit {
  pub headers: Option<HashMap<String, Either<String, Vec<String>>>>,
  pub default_headers: Option<bool>,
  pub cookies: Option<Vec<String>>,
  pub emulation: Option<Either<String, EmulationOptions>>,
  pub auth: Option<String>,
  pub bearer_auth: Option<String>,
  pub basic_auth: Option<BasicAuth>,
  pub query: Option<HashMap<String, Either<String, Vec<String>>>>,
  pub protocols: Option<Vec<String>>,
  pub force_http2: Option<bool>,
  pub read_buffer_size: Option<u32>,
  pub write_buffer_size: Option<u32>,
  pub max_write_buffer_size: Option<u32>,
  pub max_frame_size: Option<u32>,
  pub max_message_size: Option<u32>,
  pub accept_unmasked_frames: Option<bool>,
  pub proxy: Option<ProxyConfig>,
  pub local_address: Option<String>,
  pub interface: Option<String>,
}

#[derive(Default)]
pub struct ParsedRequest {
  pub method: Option<Method>,
  pub request: Request,
}

#[derive(Default)]
pub struct ParsedWebSocketRequest {
  pub request: WebSocketRequest,
}

impl RequestInit {
  pub fn parse(self) -> NapiResult<ParsedRequest> {
    let mut request = Request::default();

    let method = fill_request(&mut request, self)?;

    Ok(ParsedRequest { method, request })
  }
}

impl WebSocketInit {
  pub fn parse(self) -> NapiResult<ParsedWebSocketRequest> {
    let mut request = WebSocketRequest::default();

    if let Some(headers) = self.headers {
      request.headers = Some(convert_header_map(headers)?);
    }

    if let Some(emulation) = parse_optional_emulation(self.emulation)? {
      request.emulation = Some(emulation);
    }

    if let Some(default_headers) = self.default_headers {
      request.default_headers = Some(default_headers);
    }

    if let Some(cookies) = self.cookies {
      request.cookies = Some(convert_cookies(cookies)?);
    }

    if let Some(auth) = self.auth {
      request.auth = Some(auth);
    }

    if let Some(bearer) = self.bearer_auth {
      request.bearer_auth = Some(bearer);
    }

    if let Some(basic) = self.basic_auth {
      request.basic_auth = Some((basic.username, basic.password));
    }

    if let Some(query) = self.query {
      request.query = Some(convert_pairs(query)?);
    }

    if let Some(protocols) = self.protocols {
      request.protocols = Some(protocols);
    }

    if let Some(force_http2) = self.force_http2 {
      request.force_http2 = Some(force_http2);
    }

    request.read_buffer_size = self.read_buffer_size.map(|v| v as usize);
    request.write_buffer_size = self.write_buffer_size.map(|v| v as usize);
    request.max_write_buffer_size = self.max_write_buffer_size.map(|v| v as usize);
    request.max_frame_size = self.max_frame_size.map(|v| v as usize);
    request.max_message_size = self.max_message_size.map(|v| v as usize);
    request.accept_unmasked_frames = self.accept_unmasked_frames;

    if let Some(proxy) = self.proxy {
      request.proxy = Some(parse_proxy(proxy)?);
    }

    if let Some(local) = self.local_address {
      request.local_address = Some(parse_ip(local)?);
    }

    if let Some(interface) = self.interface {
      request.interface = Some(interface);
    }

    Ok(ParsedWebSocketRequest { request })
  }
}

fn fill_request(request: &mut Request, init: RequestInit) -> NapiResult<Option<Method>> {
  let RequestInit {
    method,
    headers,
    default_headers,
    cookies,
    emulation,
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
    timeout,
    read_timeout,
    version,
    proxy,
    local_address,
    interface,
  } = init;

  let parsed_method = method.map(|value| parse_method(&value)).transpose()?;

  if let Some(emulation) = parse_optional_emulation(emulation)? {
    request.emulation = Some(emulation);
  }

  if let Some(headers) = headers {
    request.headers = Some(convert_header_map(headers)?);
  }

  if let Some(default_headers) = default_headers {
    request.default_headers = Some(default_headers);
  }

  if let Some(cookies) = cookies {
    request.cookies = Some(convert_cookies(cookies)?);
  }

  request.allow_redirects = allow_redirects;
  request.max_redirects = max_redirects.map(|value| value as usize);
  request.gzip = gzip;
  request.brotli = brotli;
  request.deflate = deflate;
  request.zstd = zstd;
  request.auth = auth;
  request.bearer_auth = bearer_auth;
  request.basic_auth = basic_auth.map(|basic| (basic.username, basic.password));

  if let Some(query) = query {
    request.query = Some(convert_pairs(query)?);
  }

  if let Some(form) = form {
    request.form = Some(convert_pairs(form)?);
  }

  request.json = json;

  if let Some(body) = body {
    request.body = Some(match body {
      Either::A(text) => wreq::Body::from(text),
      Either::B(buffer) => wreq::Body::from(buffer.as_ref().to_vec()),
    });
  }

  request.timeout = timeout.map(duration_from_millis);
  request.read_timeout = read_timeout.map(duration_from_millis);

  if let Some(version) = version {
    request.version = Some(parse_version(&version)?);
  }

  if let Some(proxy) = proxy {
    request.proxy = Some(parse_proxy(proxy)?);
  }

  if let Some(local) = local_address {
    request.local_address = Some(parse_ip(local)?);
  }

  if let Some(interface) = interface {
    request.interface = Some(interface);
  }

  Ok(parsed_method)
}

pub(crate) fn convert_header_map(
  headers: HashMap<String, Either<String, Vec<String>>>,
) -> NapiResult<HeaderMap> {
  let mut map = HeaderMap::new();
  for (name, value) in headers {
    let header_name = HeaderName::from_bytes(name.as_bytes())
      .map_err(|err| napi_invalid(format!("invalid header name {name:?}: {err}")))?;

    match value {
      Either::A(value) => {
        let value = HeaderValue::from_str(&value)
          .map_err(|err| napi_invalid(format!("invalid header value for {name:?}: {err}")))?;
        map.insert(header_name.clone(), value);
      }
      Either::B(values) => {
        for raw in values {
          let value = HeaderValue::from_str(&raw)
            .map_err(|err| napi_invalid(format!("invalid header value for {name:?}: {err}")))?;
          map.append(header_name.clone(), value);
        }
      }
    }
  }
  Ok(map)
}

pub(crate) fn convert_cookies(cookies: Vec<String>) -> NapiResult<Vec<HeaderValue>> {
  cookies
    .into_iter()
    .map(|cookie| {
      HeaderValue::from_str(&cookie)
        .map_err(|err| napi_invalid(format!("invalid cookie value {cookie:?}: {err}")))
    })
    .collect()
}

pub(crate) fn convert_pairs(
  map: HashMap<String, Either<String, Vec<String>>>,
) -> NapiResult<Vec<(String, String)>> {
  let mut pairs = Vec::with_capacity(map.len());
  for (key, value) in map {
    match value {
      Either::A(value) => pairs.push((key.clone(), value)),
      Either::B(values) => {
        for value in values {
          pairs.push((key.clone(), value));
        }
      }
    }
  }
  Ok(pairs)
}

pub(crate) fn parse_method(method: &str) -> NapiResult<Method> {
  Method::from_bytes(method.as_bytes())
    .map_err(|err| napi_invalid(format!("invalid HTTP method: {err}")))
}

fn parse_version(version: &str) -> NapiResult<Version> {
  match version.to_ascii_uppercase().as_str() {
    "HTTP/0.9" | "0.9" => Ok(Version::HTTP_09),
    "HTTP/1.0" | "1.0" => Ok(Version::HTTP_10),
    "HTTP/1.1" | "1.1" => Ok(Version::HTTP_11),
    "HTTP/2" | "2" | "HTTP/2.0" | "2.0" => Ok(Version::HTTP_2),
    "HTTP/3" | "3" | "HTTP/3.0" | "3.0" => Ok(Version::HTTP_3),
    other => Err(napi_invalid(format!("unsupported HTTP version: {other}"))),
  }
}

pub(crate) fn parse_proxy(config: ProxyConfig) -> NapiResult<Proxy> {
  let ProxyConfig {
    uri,
    username,
    password,
  } = config;

  let mut proxy = wreq::Proxy::all(&uri).map_err(|err| napi_invalid(err.to_string()))?;
  if let Some(username) = username {
    let password = password.unwrap_or_default();
    proxy = proxy.basic_auth(&username, &password);
  }

  Ok(proxy)
}

pub(crate) fn parse_ip(value: String) -> NapiResult<IpAddr> {
  value
    .parse::<IpAddr>()
    .map_err(|err| napi_invalid(format!("invalid ip address {value:?}: {err}")))
}

pub(crate) fn duration_from_millis(value: u32) -> Duration {
  Duration::from_millis(value as u64)
}

pub(crate) fn napi_invalid(message: String) -> NapiError {
  NapiError::new(Status::InvalidArg, message)
}
