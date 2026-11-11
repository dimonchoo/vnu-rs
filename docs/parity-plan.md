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

After stage 3 (2026-11-11): strict parity 5682/5824 (97.56%, +18, no new failures),
oracle corpus 81/82 on the stage-2 files (only `href_space` left, a datatype detail);
with 17 RELAX NG files added, 98/99.

## Why it differs today

- Fixed in stage 2: tree construction. HTML now goes through a port of nu htmlparser's
  TreeBuilder (`crates/vnu-parser/src/treebuilder`); XHTML and SVG still use the
  hand-made stack in `engine/html_stack.rs`, because Java parses them as XML.
- Fixed in stage 3: content models. HTML documents are validated against the RELAX NG
  schema like Java. The hand-written content-model rules (~2.8k lines) still run; rule
  messages that repeat a schema message of the same event are dropped
  (`engine/dispatch.rs`, `drop_duplicates`) until stage 4 deletes them.
- Datatypes are still hand-written: in the schema only `w:string` and `w:mime-type`
  are real, every other `w:` datatype accepts any string.

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
3. [x] RELAX NG (`c1bc462`, `54e7de9`, `5ef164c`, `9c86f63`).
   - `crates/vnu-relaxng`: port of Jing's derivative validator (v.Nu fork of
     `jing-trang/mod/rng-validate`: PatternBuilder/interner, ValidatorPatternBuilder,
     PatternMemo, the derivative functions, RequiredContent, FindElementFunction,
     PatternValidator with NameRecordingVisitor and its exceptions), SchemaBuilderImpl and
     a compact-syntax parser, and a binary form of the expanded schema.
   - `crates/vnu-parser/schema/`: upstream `schema/` files, unchanged (see its README).
     `build.rs` compiles `html5-all.rnc` (what v.Nu uses for `text/html`: preset 3,
     HTML + SVG + MathML + RDFa; `html5full-rdfa.rnc` is generated like `build.py`) and
     the result is embedded with `include_bytes!` (~80 KB).
   - `schema/relaxng`: tree-builder pushes, pops and text go through the same filters as
     in Java (nu `AttributeName` naming with `lang` as `xml:lang`, adjusted SVG/MathML
     names, no `xmlns` attributes; `AttributesPermutingXMLReaderWrapper`; the
     custom-element, template, role, xml:lang and data-* wrappers, the role wrapper's
     messages included). Text is buffered until the next tag or comment. Wording is
     `MessageEmitterAdapter.rngMessageText`; `eval_parity` compares the
     `getDisplayMessage` wording (`messages::display_wording`) like TestRunner.
   - Also: `TreeBuilder.checkAttributes` (xmlns errors) ported so they precede schema
     errors; `main` inside `Assertions.PROHIBITED_MAIN_ANCESTORS`.
   - Not done: XHTML and SVG documents still skip the schema (Java parses them as XML
     and uses `xhtml5-all.rnc` / `svg-xhtml5-rdf-mathml.rnc`). Text errors are located
     at column 1 of the text's line. `AriaAttributeDroppingContentHandlerWrapper` drops
     only valueless `aria-*` in Java (`== ""` on the interned string); the tree events
     do not say an attribute had no value, so nothing is dropped.
4. [ ] Datatypes through the schema (Html5DatatypeException wording), then delete the
   hand-written content-model rules the schema now covers. Keep Assertions.java ports.
   What it needs, from stage 3:
   - Port `nu.validator.datatype` (`Html5DatatypeLibrary`, ~50 `w:` types) into
     `schema/relaxng/datatypes`, replacing `AnyStringDatatype`; `DatatypeException` has
     `message` and `warning` but no segments yet (`emitDatatypeErrors` renders them).
     The URL types need galimatias (`href_space`). Then the XSD types used by SVG and
     MathML (`BuiltinDatatypeLibraryFactory` treats them as strings).
   - Once schema values carry details, `messages::display_wording` must drop them for
     bad values and bad character content (`getDisplayMessage` has none).
   - Java's datatype error map is a `HashMap`: several details come in hash order.
   - Remove the hand-written content-model and datatype rules the schema covers, then
     `drop_duplicates`. The data-* name and `xml:lang` errors of the wrappers are still
     hand-written rules.
5. [ ] Benchmark (509-page audit), rebuild Docker image `vnu-rs:latest` and
   `~/.cargo/bin/vnu`.

Each stage: strict parity must not drop, oracle corpus must improve, commit per logical
step (the repo hook demands rustfmt with `--edition 2024` style and flags hot-path
allocations), push the branch, update this file.
