# Changelog

## 0.7.0 - 2026-11-15

### Breaking changes

- `Client::list_identifiers`, `list_records` and `list_sets` are no longer `async`, make no request and return the stream directly (no `Result`): the first page is fetched by the first `try_next()`. Replace `client.list_records(args).await?` with `client.list_records(args)`. A failed first request can now be retried by calling `try_next()` again, like any other page.
- `ResumableStream` returns `Err(Error::Oai(..))` for OAI-PMH errors on every page, the first included, except `noRecordsMatch` on the first page, which ends the stream with `Ok(None)` (an empty list). Previously a first-page error was returned as a normal response.

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
