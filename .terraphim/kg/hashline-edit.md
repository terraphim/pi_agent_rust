# hashline_edit

Precise-edit tool using `LINE#HASH` anchors emitted by `read`/`grep` with `hashline=true`. Avoids ambiguous string-match edits by pinning each edit to a line number + content hash, so concurrent drift can't silently mis-apply. One of the 8 built-in tools in `src/tools.rs`.

**Key files:** `src/tools.rs`

Related: tool

synonyms:: hashline_edit, hashline, line hash, hash line, precise edit, line hash edit, hash anchor, LINE#HASH, hashline edit tool, HashlineEditTool
