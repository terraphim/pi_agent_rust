# Conformance

Fixture-based conformance tests validating tool/provider/session behaviour against JSON fixtures (`tests/conformance.rs`, `tests/installer_regression.sh`). Fixture shape: `{version, tool, cases:[{name, setup, input, expected:{content_contains, content_regex, details_exact}}]}`. Run with `cargo test conformance`. Extension conformance lives under `tests/ext_conformance/`. Installer/skill integrity is gated by `scripts/skill-smoke.sh`.

**Key files:** `tests/conformance.rs`, `tests/installer_regression.sh`, `scripts/skill-smoke.sh`

Related: tool, extension, session

synonyms:: conformance, conformance test, fixture, fixture test, conformance suite, installer regression, skill smoke, ext_conformance, test fixture, content_contains, details_exact
