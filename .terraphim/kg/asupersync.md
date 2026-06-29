# asupersync

Structured-concurrency async runtime from the sibling `../asupersync` crate. Powers HTTP client, TLS (webpki-roots), SQLite, cancellation, and explicit capability context (Cx). Enables deterministic testing via LabRuntime. pi pins `asupersync = "0.3.4"`. The runtime-parking fix (#107) relies on it to idle `--acp` without CPU spin.

**Key files:** `Cargo.toml`, `src/http/client.rs`

Related: provider, sse, session

synonyms:: asupersync, async runtime, structured concurrency, runtime, labruntime, capability context, cx, http client, tls, webpki roots, cancellation, runtime parking
