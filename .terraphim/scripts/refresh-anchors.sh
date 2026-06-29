#!/usr/bin/env bash
# .terraphim/scripts/refresh-anchors.sh
#
# Refresh the pi_agent_rust terraphim-grep KG with structurally-derived code
# anchors, using ast-grep. Repeatable + idempotent: run it whenever the code
# changes to keep the thesaurus aligned with real identifiers.
#
# What it does:
#   1. ast-grep extracts pub struct / pub enum / pub trait names and
#      `impl Provider for X` targets from src/.
#   2. Identifiers are bucketed onto KG concepts by curated keyword rules.
#   3. New anchors are APPENDED to each .terraphim/kg/*.md `synonyms::` line
#      (idempotent — existing synonyms are never duplicated or removed).
#   4. .terraphim/thesaurus.json is regenerated from the (enriched) KG.
#
# Requirements: ast-grep (sg) on PATH, python3.
#
# Usage:
#   .terraphim/scripts/refresh-anchors.sh        # from repo root
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
KG_DIR="$REPO_ROOT/.terraphim/kg"
THESAURUS="$REPO_ROOT/.terraphim/thesaurus.json"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

cd "$REPO_ROOT"
command -v ast-grep >/dev/null || { echo "ast-grep not found on PATH" >&2; exit 1; }

echo ">> extracting structural identifiers with ast-grep ..."
ast-grep run -l Rust -p 'pub struct $NAME { $$$BODY }' src --json=compact > "$TMP/structs.json" 2>/dev/null || true
ast-grep run -l Rust -p 'pub enum $NAME { $$$BODY }'   src --json=compact > "$TMP/enums.json"   2>/dev/null || true
ast-grep run -l Rust -p 'pub trait $NAME { $$$BODY }'  src --json=compact > "$TMP/traits.json"  2>/dev/null || true
ast-grep run -l Rust -p 'impl $T for $NAME { $$$BODY }' src --json=compact > "$TMP/impls.json"  2>/dev/null || true

python3 - "$TMP" "$KG_DIR" "$THESAURUS" <<'PY'
import json, os, sys, collections
tmp, kg_dir, thesaurus_path = sys.argv[1], sys.argv[2], sys.argv[3]

def names(path, var):
    try: d = json.load(open(path))
    except FileNotFoundError: return []
    out = []
    for m in d:
        s = (m.get("metaVariables") or {}).get("single", {})
        n = s.get(var)
        t = n.get("text", "") if isinstance(n, dict) else ""
        if t: out.append((t, m.get("file", "")))
    return out

items = []
items += names(f"{tmp}/structs.json", "NAME")
items += names(f"{tmp}/enums.json", "NAME")
items += names(f"{tmp}/traits.json", "NAME")
# impl Provider for X -> capture the concrete type
try:
    for m in json.load(open(f"{tmp}/impls.json")):
        s = (m.get("metaVariables") or {}).get("single", {})
        T = s.get("T"); T = T.get("text","") if isinstance(T, dict) else ""
        N = s.get("NAME"); N = N.get("text","") if isinstance(N, dict) else ""
        if T == "Provider" and N: items.append((N, "src/providers/"))
except FileNotFoundError:
    pass

# Curated concept-bucketing rules (name, file) -> concept stem.
# Selective by design: name-based patterns only. Avoid file-wide catch-alls
# (e.g. f.startswith("src/providers/")) — they flood the thesaurus with
# peripheral request/response types and metrics noise.
rules = {
    "provider":        lambda n,f: n.endswith("Provider"),
    "session":         lambda n,f: "Session" in n,
    "extension":       lambda n,f: "Hostcall" in n or n.startswith("Extension") or "Capability" in n,
    "model-registry":  lambda n,f: "Model" in n,
    "sse":             lambda n,f: "Sse" in n or n == "StreamEvent",
    "acp":             lambda n,f: "Acp" in n,
    "auth":            lambda n,f: "Auth" in n or "OAuth" in n,
    "interactive-tui": lambda n,f: ("Picker" in n or "Selector" in n or n == "PiApp") and "src/interactive" in f,
    "tool":            lambda n,f: n.endswith("Tool") or n == "ToolRegistry",
    "hashline-edit":   lambda n,f: n == "HashlineEditTool",
}

buckets = collections.defaultdict(list)
seen = collections.defaultdict(set)
for n, f in items:
    if not (3 <= len(n) <= 40):
        continue
    for concept, fn in rules.items():
        if fn(n, f) and n not in seen[concept]:
            buckets[concept].append(n); seen[concept].add(n)

# Append new anchors to each concept's synonyms:: line (idempotent).
enriched = 0
for concept, anchors in buckets.items():
    path = os.path.join(kg_dir, f"{concept}.md")
    if not os.path.exists(path):
        continue
    lines = open(path, encoding="utf-8").read().splitlines()
    out, done = [], False
    for line in lines:
        if line.strip().lower().startswith("synonyms::") and not done:
            existing = line.split("::", 1)[1]
            have = {t.strip().lower() for t in existing.split(",")}
            new = [a for a in anchors if a.lower() not in have]
            out.append(line.rstrip() + ("" if not new else ", " + ", ".join(new)))
            enriched += len(new)
            done = True
        else:
            out.append(line)
    if done:
        open(path, "w", encoding="utf-8").write("\n".join(out) + "\n")
print(f">> appended {enriched} new structural anchors across {len(buckets)} concepts")

# Regenerate thesaurus.json from the (now enriched) KG markdown.
data, cid = {}, 100
for f in sorted(os.listdir(kg_dir)):
    if not f.endswith(".md"):
        continue
    nterm = os.path.splitext(f)[0]
    txt = open(os.path.join(kg_dir, f), encoding="utf-8").read()
    syn = next((l.split("::", 1)[1] for l in txt.splitlines()
                if l.strip().lower().startswith("synonyms::")), "")
    for t in dict.fromkeys([nterm] + [s.strip().lower() for s in syn.split(",") if s.strip()]):
        data.setdefault(t.lower(), {"id": cid, "nterm": nterm})
    cid += 1
json.dump({"name": "Pi Agent Rust Engineer", "data": data},
          open(thesaurus_path, "w"), indent=2, ensure_ascii=False)
print(f">> regenerated {thesaurus_path}: {len(data)} thesaurus entries")
PY

echo ">> done. verify with:"
echo "   terraphim-grep 'ReadTool' --paths src --thesaurus .terraphim/thesaurus.json -n 3 --json"
