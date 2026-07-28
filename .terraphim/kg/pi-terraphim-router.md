# pi_terraphim_router

Optional knowledge-graph model router (`src/pi_terraphim_router/`, ~1495 lines: mod/rpc_client/types/error). Feature-gated behind `terraphim-routing` (off by default). Uses a KgRouter with Aho-Corasick concept matching + readiness-aware fallback to pick a provider/model for a prompt. **Currently BLOCKED** on `task/terraphim-router-blocked`: its path deps (`terraphim_router/types/automata`) were extracted to the standalone terraphim-core repo (registry deps, Gitea #1910) and need migrating.

**Key files:** `src/pi_terraphim_router/`, branch `task/terraphim-router-blocked`, `examples/terraphim_router.rs`

Related: model-registry, provider, auth

synonyms:: pi_terraphim_router, terraphim router, kg router, model routing, intelligent model selection, demoroute, demo-route, readiness-aware routing, terraphim-routing, terraphim-routing feature, aho-corasick router
