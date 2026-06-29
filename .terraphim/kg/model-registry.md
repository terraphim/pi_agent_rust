# ModelRegistry

Model resolution layer (`src/models.rs`, `src/app.rs`). Resolves `ModelEntry` records from built-in seeds, a legacy generated catalog, and a user `models.json` (with env/file value resolution). Upstream commit #117 made the data catalog authoritative for reasoning effort + thinking-control metadata. `PROVIDER_DEFAULT_MODELS` in `src/app.rs` lists newest-first default candidates per provider (e.g. kimi-for-coding -> kimi-k2.7, kimi-k2.6, ...). Autocomplete candidates drive the `/model` picker.

**Key files:** `src/models.rs`, `src/app.rs`, `docs/provider-upstream-model-ids-snapshot.json`

Related: provider, interactive-tui, auth

synonyms:: model registry, modelregistry, model entry, modelentry, models.json, model catalog, built-in models, provider default models, model resolution, autocomplete, reasoning effort, thinking control, default model, model id, model fetch, kimi-k2.7, glm-5.2, RpcModelInfo, RpcCycleModelResult, ModelKey, ModelSelectorOverlay, RpcScopedModel, ScopedModel, ModelSelection, Model, ModelCost, ModelRoutingEvidence, ModelsConfig, ModelConfig, ModelAutocompleteCandidate, ModelChangeEntry, RuntimeRiskBaselineModel
