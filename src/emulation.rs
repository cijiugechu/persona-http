use napi::bindgen_prelude::{Either, Result as NapiResult};
use napi::{Error as NapiError, Status};
use napi_derive::napi;
use strum::VariantArray;
use wreq_util::{Emulation, EmulationOS, EmulationOption};

#[napi(object)]
pub struct EmulationOptions {
  pub preset: Option<String>,
  pub os: Option<String>,
  #[napi(js_name = "skipHttp2")]
  pub skip_http2: Option<bool>,
  #[napi(js_name = "skipHeaders")]
  pub skip_headers: Option<bool>,
}

pub(crate) type EmulationInput = Either<String, EmulationOptions>;

pub(crate) fn parse_optional_emulation(
  option: Option<EmulationInput>,
) -> NapiResult<Option<EmulationOption>> {
  option.map(parse_emulation).transpose()
}

pub(crate) fn parse_emulation(option: EmulationInput) -> NapiResult<EmulationOption> {
  match option {
    Either::A(preset) => build_emulation(EmulationOptions {
      preset: Some(preset),
      os: None,
      skip_http2: None,
      skip_headers: None,
    }),
    Either::B(options) => build_emulation(options),
  }
}

fn build_emulation(options: EmulationOptions) -> NapiResult<EmulationOption> {
  let emulation = options
    .preset
    .as_deref()
    .map(parse_emulation_preset)
    .transpose()?;
  let emulation_os = options.os.as_deref().map(parse_emulation_os).transpose()?;

  let skip_http2 = options.skip_http2.unwrap_or(false);
  let skip_headers = options.skip_headers.unwrap_or(false);

  Ok(
    EmulationOption::builder()
      .emulation(emulation.unwrap_or_default())
      .emulation_os(emulation_os.unwrap_or_default())
      .skip_http2(skip_http2)
      .skip_headers(skip_headers)
      .build(),
  )
}

fn parse_emulation_preset(value: &str) -> NapiResult<Emulation> {
  let trimmed = value.trim();
  if trimmed.is_empty() {
    return Err(invalid_arg("emulation preset cannot be empty"));
  }

  let normalized = normalize_label(trimmed);

  for candidate in Emulation::VARIANTS.iter().copied() {
    let candidate_label = format!("{candidate:?}");
    if normalize_label(&candidate_label) == normalized {
      return Ok(candidate);
    }
  }

  Err(invalid_arg(format!(
    "unsupported emulation preset: {value}. For example, try chrome_140 or firefox_135"
  )))
}

fn parse_emulation_os(value: &str) -> NapiResult<EmulationOS> {
  let trimmed = value.trim();
  if trimmed.is_empty() {
    return Err(invalid_arg("emulation os cannot be empty"));
  }

  // Common aliases so users don't have to remember exact enum casing.
  let normalized = normalize_label(trimmed);
  let alias = match normalized.as_str() {
    "mac" | "osx" => Some(EmulationOS::MacOS),
    "win" | "win32" | "win64" => Some(EmulationOS::Windows),
    "iphone" | "ipad" => Some(EmulationOS::IOS),
    _ => None,
  };
  if let Some(alias) = alias {
    return Ok(alias);
  }

  for candidate in EmulationOS::VARIANTS.iter().copied() {
    let candidate_label = format!("{candidate:?}");
    if normalize_label(&candidate_label) == normalized {
      return Ok(candidate);
    }
  }

  Err(invalid_arg(format!(
    "unsupported emulation os: {value}. Accepted values include windows, macos, linux, android, ios"
  )))
}

fn normalize_label(value: &str) -> String {
  value
    .chars()
    .filter(|c| c.is_ascii_alphanumeric())
    .map(|c| c.to_ascii_lowercase())
    .collect()
}

fn invalid_arg(message: impl Into<String>) -> NapiError {
  NapiError::new(Status::InvalidArg, message.into())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_simple_preset() {
    let preset = parse_emulation_preset("chrome_105").unwrap();
    assert_eq!(preset, Emulation::Chrome105);
  }

  #[test]
  fn parses_camel_case_preset() {
    let preset = parse_emulation_preset("SafariIos17_4_1").unwrap();
    assert_eq!(preset, Emulation::SafariIos17_4_1);
  }

  #[test]
  fn parses_os_alias() {
    let cases = [
      ("osx", EmulationOS::MacOS),
      ("macos", EmulationOS::MacOS),
      ("windows", EmulationOS::Windows),
      ("win", EmulationOS::Windows),
      ("iphone", EmulationOS::IOS),
      ("ipad", EmulationOS::IOS),
      ("ios", EmulationOS::IOS),
      ("android", EmulationOS::Android),
      ("linux", EmulationOS::Linux),
      ("mac", EmulationOS::MacOS),
      ("win32", EmulationOS::Windows),
      ("win64", EmulationOS::Windows),
    ];

    for (input, expected) in cases {
      let os = parse_emulation_os(input).unwrap();
      assert_eq!(os, expected);
    }
  }
}
