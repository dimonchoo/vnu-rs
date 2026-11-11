# Logic parity with Java v.Nu

Goal: vnu-rs reports the same errors, warnings and wording as Java v.Nu for any input,
not only for the upstream test files. Work happens on `feat/relaxng-schema` and branches
off it, one stage per session.

## How to measure

- `make parity`: upstream tests with upstream TestRunner semantics (strict). `--lenient`
  keeps the old substring match.
- `make oracle-up && make oracle && make oracle-down`: diff against real Java v.Nu
  (Docker, port 8899) on `crates/vnu-oracle/corpus/*.html`. Add a corpus file for every
  bug you fix. Port 8888 serves vnu-rs itself, never use it as the oracle.

Baseline (2026-11-11): strict parity 5658/5824 (97.15%), oracle corpus 19/36.

After stage 2 (2026-11-11): strict parity 5664/5824 (97.25%, no new failures), oracle
corpus 26/36 on the original files; with 46 tree-construction files added, 72/82. The
10 remaining diffs are RELAX NG content-model and datatype messages (stages 3 and 4).

## Why it differs today

- Fixed in stage 2: tree construction. HTML now goes through a port of nu htmlparser's
  TreeBuilder (`crates/vnu-parser/src/treebuilder`); XHTML and SVG still use the
  hand-made stack in `engine/html_stack.rs`, because Java parses them as XML.
- Content models are ~2.8k lines of hand-written special cases (figure, dl, details,
  colgroup, `rules/content_model.rs`). Java validates against the RELAX NG schema
  (`validator-upstream/schema/html5/*.rnc`) with Jing, so anything not special-cased
  passes ("div" in "ul" or "span", text in "ul", "p" in "dl", nested "main",
  unknown attributes).

## Stages

1. [x] Honest metrics: strict `eval_parity`, `oracle_diff` against Java (`7b93823`).
2. [x] Tree construction like nu htmlparser (`665ffe1`, `0c18f82`). TreeBuilder is
   ported with SAXStreamer semantics: foster parenting, adoption-agency reparenting
   and attributes added to `html`/`body` report "Cannot recover after last error. Any
   further errors will be ignored." and nothing after that token is reported (no rules,
   no end-of-document checks, the syntax rescan stops there). Rules get start/end
   events from the builder's pushes and pops, so implied `html`/`head`/`body`/`tbody`
   and optional end tags match Java's checkers. The builder also picks the tokenizer
   state (raw text, CDATA in foreign content). Not ported: fragment parsing,
   `checkAttributes` xmlns errors, `checkPopName` XML 1.0 warnings (filtered upstream),
   meta charset re-encoding. Tokenizer errors still come from html5ever plus the
   `syntax/` rescan, not from nu's Tokenizer.
3. [ ] RELAX NG: parse `schema/html5/*.rnc` at build time (embed the result), port Jing's
   derivative validator (`jing-trang/mod/rng-validate`, ~6.6k lines) and its messages
   (`resources/Messages.properties` plus v.Nu wording "Element “X” not allowed as child of
   element “Y” in this context. (Suppressing further errors from this subtree.)").
4. [ ] Datatypes through the schema (Html5DatatypeException wording), then delete the
   hand-written content-model rules the schema now covers. Keep Assertions.java ports.
5. [ ] Benchmark (509-page audit), rebuild Docker image `vnu-rs:latest` and
   `~/.cargo/bin/vnu`.

Each stage: strict parity must not drop, oracle corpus must improve, commit per logical
step (the repo hook demands rustfmt with `--edition 2024` style and flags hot-path
allocations), push the branch, update this file.
