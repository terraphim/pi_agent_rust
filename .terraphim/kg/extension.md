# Extension

JavaScript extension runtime on QuickJS with a capability policy layer. Extensions provide extra tools/providers via hostcalls bridged through `src/extensions_js.rs`. The policy engine (`src/extensions.rs`) enforces allow/deny capability controls (filesystem, network, subprocess). Conformance fixtures under `tests/ext_conformance/` validate JS parity and taxonomy. The `wasm-host` feature gates a Wasmtime runtime alternative.

**Key files:** `src/extensions.rs`, `src/extensions_js.rs`, `tests/ext_conformance/`

Related: tool, provider, conformance

synonyms:: extension, extensions, quickjs, quickjs runtime, javascript extension, capability policy, hostcall, hostcalls, extension runtime, wasm host, wasmtime, extension parity, extension taxonomy, extension oauth
