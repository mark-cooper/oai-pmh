# Changelog

## 0.6.0 - 2026-11-04

### Breaking changes

- `error` on every response type is now `Vec<ResponseError>` (was `Option<ResponseError>`), since a response can carry several `<error>` elements. `is_err()` is unchanged.
- `Error` has new variants: `RateLimited`, `Oai` and `ParseMismatch`.
- `Record.about` is no longer populated and is always empty. `<about>` holds arbitrary XML, which previously made the whole response fail to parse.
- `ResumableResponse` now requires `FromXml`. Its `from_xml` and `extract_token` methods are gone, and `errors()` and `into_errors()` are added.
- `Client::new` now applies timeouts: 10s to connect and 300s per request. Previously there were none.
- `ResumableStream` now returns `Err(Error::Oai(..))` when a page after the first carries OAI-PMH errors. Previously the stream ended silently, which could look like a complete harvest. A first-page error is still returned as a normal response.

### Added

- `Client::with_http_client` to supply your own `reqwest::Client` (timeouts, proxy, TLS).
- `Error::RateLimited { status, retry_after }` for HTTP 503 and 429. `retry_after` is read from `Retry-After` when it is given in seconds. See `examples/retry.rs`.
- `FromXml` trait in `client::response`.

### Fixed

- `<error>` elements with no message text, such as `<error code="noRecordsMatch"/>`, no longer fail to parse.
- A failed page fetch in a `ResumableStream` keeps its resumption token, so calling `try_next()` again retries the same page. Previously the stream ended.
- `ListRecords` returns `ParseMismatch` instead of silently mismatching metadata if its record and metadata counts differ.

### Changed

- Default request headers are built once per client.
- Updated dependencies, including `quick-xml` 0.42 and `reqwest` 0.13.5.
