//! Parses the vendored HTML RELAX NG schema (`schema/`, see its README) and writes the
//! compiled form to `$OUT_DIR/html5.rngc` for `include_bytes!`. The root is
//! `http://s.validator.nu/html5-all.rnc`, the schema v.Nu uses for `text/html`.

use std::path::{Path, PathBuf};

/// `schemaDriverBase` and friends from upstream `build/build.py`.
const HTML5FULL: &str = r#"default namespace = "http://www.w3.org/1999/xhtml"
include "common.rnc" {
        XMLonly = notAllowed
        HTMLonly = empty
        v5only = empty
        nonHTMLizable = notAllowed
        nonRoundtrippable = notAllowed
}
start = html.elem
include "phrase.rnc"
include "block.rnc"
include "sectional.rnc"
include "revision.rnc"
include "embed.rnc"
include "core-scripting.rnc"
include "structural.rnc"
include "ruby.rnc"
include "media.rnc"
include "tables.rnc"
include "form-datatypes.rnc"
include "web-forms.rnc"
include "web-forms2.rnc"
include "web-components.rnc"
include "applications.rnc"
include "data.rnc"
include "legacy.rnc"
include "aria.rnc"
include "meta.rnc" {
        html.inner =
            (    head.elem
            ,    (    body.elem
                 |    frameset.elem
                 )
            )
}
include "its20-html5.rnc"
common.attrs &= its-html-attributes
include "microdata.rnc"
"#;

/// Where a file of upstream's `build/schema` comes from in `schema/`.
fn source_path(schema_dir: &Path, build_path: &str) -> Option<PathBuf> {
    let mapped = match build_path {
        "html5-all.rnc" | "html5-svg-mathml.rnc" => format!(".drivers/{build_path}"),
        "html5/legacy.rnc" => ".drivers/legacy.rnc".to_string(),
        "html5/its20-html5.rnc" | "html5/its20-html5-types.rnc" => {
            format!("its2/{}", &build_path["html5/".len()..])
        }
        other => other.to_string(),
    };
    let path = schema_dir.join(mapped);
    path.is_file().then_some(path)
}

/// Resolves `href` against `base` (both `build/schema`-relative paths).
fn resolve(href: &str, base: &str) -> String {
    let mut parts: Vec<&str> = match base.rfind('/') {
        Some(i) => base[..i].split('/').collect(),
        None => Vec::new(),
    };
    for seg in href.split('/') {
        match seg {
            "." | "" => {}
            ".." => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    parts.join("/")
}

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let schema_dir = manifest.join("schema");
    println!("cargo:rerun-if-changed=schema");
    println!("cargo:rerun-if-changed=build.rs");

    let loader = |href: &str, base: &str| -> Result<(String, String), String> {
        let uri = resolve(href, base);
        // buildSchemaDriverHtml5RDFa: html5full.rnc plus rdfa.rnc.
        if uri == "html5/html5full-rdfa.rnc" {
            return Ok((uri, format!("{HTML5FULL}include \"rdfa.rnc\"\n")));
        }
        let path = source_path(&schema_dir, &uri).ok_or_else(|| format!("no schema file {uri}"))?;
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok((uri, text))
    };
    // VerifierServletTransaction.HTML5_SCHEMA: preset 3, "HTML + SVG + MathML + RDFa".
    let schema = vnu_relaxng::Schema::parse("html5-all.rnc", &loader)
        .unwrap_or_else(|e| panic!("cannot compile the HTML schema:\n{e}"));
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("html5.rngc");
    std::fs::write(&out, schema.to_bytes().expect("serialize the schema"))
        .expect("write compiled schema");
}
