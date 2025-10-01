# PyO3 Binding Overview and napi-rs Migration Plan

This note captures the important pieces of the current PyO3-based bindings under `py/` and outlines the lowest-risk path for porting the bindings to [`napi-rs`](https://napi.rs/).

## 1. Current Binding Layout
- `py/src/lib.rs`: entry point that registers every class and free function exposed to Python (`get`, `post`, etc.) and wires submodules (`http1`, `http2`, `tls`, `header`, `cookie`, `emulation`, `blocking`, `exceptions`).
- `py/src/bridge/`: adapters between Python's asyncio loop and the shared Tokio runtime. `Runtime` wraps `tokio::runtime::Runtime`, `future_into_py` turns Rust futures into awaitables, and `sync.rs`/`task.rs` handle cancellation, contextvars, and cross-thread messaging.
- `py/src/client/`: the main HTTP/WebSocket API. Key pieces:
  - `mod.rs`: defines the `Client` / `BlockingClient` surface, a rich builder extracted from Python kwargs, and helper `execute_request` / `execute_websocket_request` functions.
  - `body/`: turns Python strings/bytes/iterables into `wreq::Body`, with streaming support (`SyncStream`, `AsyncStream`).
  - `req.rs`: data carriers for request options (`Request`, `WebSocketRequest`).
  - `resp/`: response wrappers (`Response`, `BlockingResponse`, `Streamer`, WebSocket types, redirect `History`).
  - `nogil.rs`: `NoGIL` future wrapper to run async work without holding the GIL.
  - `dns.rs`: Hickory-based DNS resolver plugged into `wreq`.
- `py/src/http*/`, `tls/`, `emulation.rs`, `proxy.rs`: thin newtypes/enums mirroring `wreq` and `wreq-util` configuration objects, tagged with `#[pyclass]` / `#[pyfunction]` attributes.
- `py/src/error.rs`: declares all exported exception types and a central `Error` enum that maps internal failures to Python errors.
- `py/src/extractor.rs`: generic conversion helpers that translate Python dictionaries/lists/enums into the corresponding Rust types (`HeaderMap`, `OrigHeaderMap`, `EmulationOption`, `Proxy`, etc.).
- `py/src/buffer.rs`: zero-copy conversion of `bytes::Bytes` into Python buffers (`PyBuffer`).

The compiled module is surfaced to Python via `python/rnet/`, which mostly re-exports the Rust-backed classes.

## 2. Public API Surface (PyO3)
- **Top-level HTTP helpers** (`get`, `post`, `put`, `patch`, `delete`, `head`, `options`, `trace`, `request`): async functions calling `execute_request` and returning awaitables.
- **WebSocket helper** (`websocket`): async wrapper over `execute_websocket_request`.
- **`Client` / `BlockingClient`**: expose per-instance methods mirroring the free helpers plus configuration via keyword arguments (`Client::new(**kwds)`). Builder supports emulation, headers, cookies, timeouts, pool tuning, protocol selection, TLS, proxies, compression, TCP tuning.
- **`Response` / `BlockingResponse`**: provide accessors (`status`, `version`, `headers`, `cookies`, `history`, `peer_certificate`), body readers (`text`, `json`, `bytes`, `stream`), and context-manager semantics.
- **WebSocket types**: `WebSocket`, `BlockingWebSocket`, `Message` with helpers for JSON/text/binary and command-channel-based send/recv/close operations.
- **Configuration types**: `Http1Options`, `Http2Options`, `TlsOptions`, TLS enums (`TlsVersion`, `AlpnProtocol`, etc.), HTTP enums (`Method`, `Version`), header/cookie wrappers, proxy factories, emulation enums/options.
- **Exceptions**: mapped 1:1 from `wreq::Error` categories (`RequestError`, `TimeoutError`, etc.) plus runtime errors (`RustPanic`, `DNSResolverError`).

## 3. Supporting Infrastructure
- **Tokio runtime bridge**: `Runtime::future_into_py` + `NoGIL` ensure async work runs on a shared multi-threaded runtime while releasing the GIL. Cancellation propagates through `Cancellable`, `PyDoneCallback`, and Python `asyncio.Future` callbacks.
- **Extractor layer**: centralizes Python→Rust conversions and is heavily tied to `PyAny` / `PyDict` APIs.
- **Zero-copy buffers**: `PyBuffer` exposes response bodies and header values without reallocating.
- **Streaming bodies**: request bodies accept sync/async iterables by wrapping them into Rust `Stream`s via runtime adapters.

## 4. Data Flow Summary
1. Python code calls a free function or a `Client` method.
2. Builder/extractor modules translate kwargs into `wreq` builders (`ClientBuilder`, `RequestBuilder`).
3. Operations run on the shared Tokio runtime; futures are turned into Python awaitables (or executed synchronously for blocking variants).
4. Responses/newtypes wrap `wreq` types and expose ergonomic Python methods, maintaining the ability to re-read bodies by caching them in `ArcSwapOption`.
5. Errors bubble through the central `Error` enum to unified exception classes.

## 5. Migration Considerations for napi-rs
- **Attribute macros**: `#[pyclass]`, `#[pymethods]`, `#[pyfunction]`, `#[pymodule]` will need napi equivalents (`#[napi]`, `#[module_exports]`, `napi::Result`). The Rust structs already isolate most logic, but constructors/methods depend on PyO3 glue (`Python`, `PyBackedStr`, `PyResult`).
- **Runtime bridge**: asyncio/GIL handling is Python-specific. For Node we can rely on `#[napi] async fn` support (which produces Promises) or custom `AsyncTask`. The shared Tokio runtime can stay, but cancellation and context propagation must integrate with Node's event loop instead of Python futures.
- **Type conversions**: `Extractor`, `PyBackedStr`, `PyBuffer`, `PyAny` interactions must be replaced with `napi::Value`, `JsObject`, `JsBuffer`, `Env` conversions. Many helper types can be re-used if we introduce binding-neutral builders that accept owned Rust data.
- **Streaming bodies**: napi offers `JsBuffer` and (with extra work) Node streams/AsyncIterables. For a first cut we can accept `Buffer`, `string`, and JSON-compatible inputs, deferring iterable streaming until later.
- **Target API**: Node users expect Promises and classes; the shape should mimic the Python API for parity (`get`, `Client`, `Response`, etc.), but naming/async style should follow JS conventions.
- **Packaging**: replace Maturin build steps with `napi-build`/`napi::bindgen`. Continuous integration will need Node toolchain and `npm` packaging.

## 6. Lowest-Resistance Migration Path
1. **Isolate binding-agnostic core**
   - Move request/response logic, builder structs, error mapping, proxy/tls/http option wrappers into a `bindings/common` module or crate with plain Rust types (no PyO3). Many structs are already thin newtypes; we can strip the `#[pyclass]` attributes by introducing wrapper types in the PyO3 layer. The common core should expose:
     - `RnetClient` (wrapping `wreq::Client`) with async methods returning plain Rust futures.
     - `RequestParams`, `WebSocketParams`, `Http1Config`, `Http2Config`, `TlsConfig`, etc., as serde-friendly/`Clone` data.
     - Response/WebSocket wrappers that deal purely in Rust types (`Bytes`, `HeaderMap`, etc.).
   - Adapt the existing PyO3 layer to consume the new core so Python behaviour stays unchanged during refactor.

2. **Design napi-friendly bindings atop the core**
   - Enable the `tokio_rt` feature in `napi` and expose async functions with `#[napi] async fn get(...) -> napi::Result<ResponseHandle>` returning Promises.
   - Implement `#[napi]` classes for `Client`, `Response`, `WebSocket`, mirroring the Python surface but using Node-friendly types (`String`, `Buffer`, `Vec<u8>`, `napi::bindgen_prelude::Either` for optional params, `JsObject` for config).
   - Provide `FromNapiValue` implementations or helper constructors that map JS objects to the core structs (similar role to `Extractor`).

3. **Re-create utility layers**
   - Replace `PyBuffer` with wrappers around `JsBuffer`/`TypedArray`. For zero-copy, convert `Bytes` via `JsBuffer::from_slice` or by sharing `Arc<[u8]>`.
   - Swap the asyncio bridge for napi's Promise machinery. For streaming responses, expose async iterators with `napi::bindgen` (e.g., implement `#[napi] pub fn stream(...) -> AsyncIterator` using `ReadableStream` shims or yield arrays/Buffers).
   - Map error enum variants to `napi::Error` with codes matching the Python exceptions.

4. **Ship incremental feature parity**
   - Phase A: core HTTP requests (`get`/`Client.request`), response body readers (`text`, `json`, `bytes`), headers, status.
   - Phase B: TLS/HTTP1/HTTP2 configuration, proxies, emulation options, cookie store.
   - Phase C: WebSocket helpers and streaming bodies.
   - Phase D: Advanced TLS permutations, DNS resolver, key logging.

5. **Tooling & distribution**
   - Add `napi-build` workflow (GitHub actions matrix for Node LTS + target triples).
   - Generate TypeScript definitions via `napi-build --dts` to match the JS API.
   - Prepare `package.json` and npm publishing scripts; keep `py/` package intact during transition.

## 7. Risks & Open Questions
- **Streaming + AsyncIterator parity**: mapping Python's iterables and WebSocket streaming to idiomatic JS may need additional buffering or Node stream bridges.
- **Resource management**: Node does not have RAII context managers; ensure we expose explicit `close()` and rely on `Finalizer` hooks for cleanup.
- **Backpressure and cancellation**: napi Promises lack built-in cancellation—additional APIs may be needed to propagate abort signals similar to `PyDoneCallback`.
- **Emulation exposure**: confirm which emulation combinations are relevant to JavaScript users and whether to keep exhaustive enums or expose a smaller curated set.

## 8. Immediate Next Steps
1. Carve out a `core` module encapsulating `Client`, `RequestParams`, `ResponseState`, `WsState`, `Error` without PyO3 types.
2. Refactor PyO3 bindings to consume the new core and validate tests still pass.
3. Prototype a minimal napi module (`#[napi] async fn get(url: String)`) that calls into the core.
4. Incrementally port configuration surfaces and response types, keeping parity lists to track missing features.

Keeping the core logic identical reduces risk: once the shared layer exists, the napi bindings become a thin translation layer much like the current PyO3 facade.
