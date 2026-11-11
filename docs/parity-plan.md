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

## Why it differs today

- Only the html5ever **tokenizer** is used, with a hand-made open-element stack.
  Java uses nu htmlparser's full tree builder, so tree-construction parse errors are
  missing ("Start tag “p” seen in “table”.", stray end tags, implied end tags).
- Content models are ~2.8k lines of hand-written special cases (figure, dl, details,
  colgroup, `rules/content_model.rs`). Java validates against the RELAX NG schema
  (`validator-upstream/schema/html5/*.rnc`) with Jing, so anything not special-cased
  passes ("div" in "ul" or "span", text in "ul", "p" in "dl", nested "main",
  unknown attributes).

## Stages

1. [x] Honest metrics: strict `eval_parity`, `oracle_diff` against Java (`7b93823`).
2. [ ] Tree construction like nu htmlparser
   (`validator-upstream/htmlparser/src/nu/validator/htmlparser/impl/TreeBuilder.java`,
   6.6k lines): emit the same parse errors with the same wording and positions. Either
   port TreeBuilder or drive html5ever's tree builder and map its errors; the wording
   must match Java exactly.
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
