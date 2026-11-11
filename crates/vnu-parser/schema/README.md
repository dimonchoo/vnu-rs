# Vendored RELAX NG schemas

Copied unchanged from `validator/validator` (`schema/`, commit
26b30abaf6fa9435e6f43bd38911add246b8c4e1, 2026-11-04), keeping its layout:

- `.drivers/html5-all.rnc`, `.drivers/html5.rnc`, `.drivers/html5-svg-mathml.rnc`,
  `.drivers/legacy.rnc`
- `html5/`, `svg11/`, `mml3/`, `its2/`

`build.rs` parses them at build time and embeds the compiled schema. It lays the files
out like upstream's `build/build.py` (`buildSchemaDrivers`) does: `.drivers/html5-all.rnc`
is the root (`http://s.validator.nu/html5-all.rnc`, preset 3 "HTML + SVG + MathML +
RDFa", which v.Nu picks for `text/html`), `legacy.rnc` and the ITS files are read as
if copied into `html5/`, and `html5/html5full-rdfa.rnc` is generated the same way
(`buildSchemaDriverHtml5NoMicrodata`, `buildSchemaDriverHtml5`,
`buildSchemaDriverHtml5RDFa`).

To update, copy the same files from a newer upstream checkout and update the commit
above. Licenses: `LICENSE` (validator, MIT) and `html5/LICENSE`.
