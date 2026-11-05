pub mod element_attrs;
pub mod element_datatypes;
pub mod media_query;
pub mod microdata;
pub mod mime;
pub mod temporal;
pub mod tokens;
pub mod urls;
pub mod usemap;

use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use element_attrs::validate_element_attributes;
use element_datatypes::validate_element_datatypes;
use media_query::is_valid_media_query_list;
use microdata::validate_microdata;
use mime::is_valid_mime_type;
use tokens::*;
use urls::validate_url_attributes;
use usemap::validate_usemap;
use vnu_core::VNuMessage;

#[inline]
pub(super) fn bad_value_error(
    val: &str,
    attr: &str,
    el: &str,
    line: usize,
    col: usize,
) -> VNuMessage {
    VNuMessage::error(
        format!(
            "Bad value “{}” for attribute “{}” on element “{}”.",
            val, attr, el
        ),
        line,
        col,
    )
}

#[inline]
pub(super) fn bad_value_warning(
    val: &str,
    attr: &str,
    el: &str,
    line: usize,
    col: usize,
) -> VNuMessage {
    VNuMessage::warning(
        format!(
            "Bad value “{}” for attribute “{}” on element “{}”.",
            val, attr, el
        ),
        line,
        col,
    )
}

#[derive(Default)]
pub struct DatatypesRule {
    autofocus_scoping_roots: std::collections::HashSet<usize>,
    pending_usemap_checks: Vec<(String, usize, usize)>,
    defined_map_names: std::collections::HashSet<String>,
    pending_itemref_checks: Vec<(String, usize, usize)>,
    all_itemref_ids: std::collections::HashSet<String>,
    pending_itemprop_checks: Vec<(usize, usize, Option<String>)>,
}

impl Rule for DatatypesRule {
    fn name(&self) -> &'static str {
        "datatypes"
    }

    fn reset(&mut self) {
        self.autofocus_scoping_roots.clear();
        self.pending_usemap_checks.clear();
        self.defined_map_names.clear();
        self.pending_itemref_checks.clear();
        self.all_itemref_ids.clear();
        self.pending_itemprop_checks.clear();
    }

    fn on_finish(&mut self, ctx: &ValidationContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        for (name, line, col) in &self.pending_usemap_checks {
            if !self.defined_map_names.contains(name) {
                msgs.push(VNuMessage::error(
                    format!(
                        "The hash-name reference in attribute “usemap” referred to “{}”, but there is no “map” element with a “name” attribute with that value.",
                        name
                    ),
                    *line,
                    *col,
                ));
            }
        }
        for (id, line, col) in &self.pending_itemref_checks {
            if !ctx.id_to_tag.contains_key(id) {
                msgs.push(VNuMessage::error(
                    format!(
                        "The “itemref” attribute referenced “{}”, but there is no element with an “id” attribute with that value.",
                        id
                    ),
                    *line,
                    *col,
                ));
            }
        }
        for (line, col, id_opt) in &self.pending_itemprop_checks {
            let referenced = id_opt
                .as_ref()
                .is_some_and(|id| self.all_itemref_ids.contains(id));
            if !referenced {
                msgs.push(VNuMessage::error(
                    "The “itemprop” attribute was specified, but the element is not a property of any item.",
                    *line,
                    *col,
                ));
            }
        }
        msgs
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let el = tag.name.to_ascii_lowercase();

        // 1. Target browsing context
        if let Some(target) = tag.get_attr("target") {
            if matches!(el.as_str(), "a" | "area" | "form" | "base")
                && !is_valid_browsing_context(target)
            {
                msgs.push(bad_value_error(target, "target", &el, tag.line, tag.col));
            }
        }

        // autofocus
        if tag.has_attr("autofocus") {
            let root_id = ctx
                .open_elements
                .iter()
                .rev()
                .find_map(|e| {
                    if e.name.eq_ignore_ascii_case("dialog") || e.has_attr("popover") {
                        Some(e.byte_offset)
                    } else {
                        None
                    }
                })
                .unwrap_or(0);

            if !self.autofocus_scoping_roots.insert(root_id) {
                msgs.push(VNuMessage::error(
                    "There must not be two elements with the same \"nearest ancestor autofocus scoping root element\" that both have the “autofocus” attribute specified.",
                    tag.line,
                    tag.col,
                ));
            }
        }

        validate_element_attributes(tag, &el, &mut msgs);

        for (attr_name, _) in &tag.attrs {
            if attr_name == "data-" {
                msgs.push(VNuMessage::error(
                    format!(
                        "Attribute “data-” not allowed on element “{}” at this point.",
                        el
                    ),
                    tag.line,
                    tag.col,
                ));
            } else if attr_name.starts_with("data-") && ctx.is_xhtml {
                if attr_name.chars().any(|c| c.is_ascii_uppercase()) {
                    msgs.push(VNuMessage::error(
                        "“data-*” attributes must not have characters from the range “A”…“Z” in the name.",
                        tag.line,
                        tag.col,
                    ));
                }
            }
            if el == "div" && attr_name == "name" {
                msgs.push(VNuMessage::error(
                    "Attribute “name” not allowed on element “div” at this point.",
                    tag.line,
                    tag.col,
                ));
            }
        }

        validate_usemap(
            ctx,
            tag,
            &el,
            &mut self.pending_usemap_checks,
            &mut self.defined_map_names,
            &mut msgs,
        );

        validate_microdata(
            ctx,
            tag,
            &mut self.all_itemref_ids,
            &mut self.pending_itemref_checks,
            &mut self.pending_itemprop_checks,
            &mut msgs,
        );

        // 2. Custom element name: is="..."
        if let Some(is_val) = tag.get_attr("is") {
            if !is_valid_custom_element_name(is_val) {
                msgs.push(bad_value_error(is_val, "is", &el, tag.line, tag.col));
            }
        }

        // 3. Hash name: usemap="..."
        if let Some(usemap) = tag.get_attr("usemap") {
            if !usemap.starts_with('#') || usemap.len() < 2 {
                msgs.push(bad_value_error(usemap, "usemap", &el, tag.line, tag.col));
            }
        }

        // 4. ID attribute: empty or contains whitespace
        if let Some(id) = tag.get_attr("id") {
            if id.is_empty() || id.chars().any(|c| c.is_ascii_whitespace()) {
                msgs.push(bad_value_error(id, "id", &el, tag.line, tag.col));
            }
        }

        // map id and name consistency
        if el == "map" {
            if let (Some(id), Some(name)) = (tag.get_attr("id"), tag.get_attr("name")) {
                if id != name {
                    msgs.push(VNuMessage::error(
                        "The “id” attribute on a “map” element must have an the same value as the “name” attribute.",
                        tag.line,
                        tag.col,
                    ));
                }
            }
        }

        // Media query list on media attribute
        if let Some(m) = tag.get_attr("media") {
            if matches!(el.as_str(), "link" | "style" | "source") && !is_valid_media_query_list(m) {
                msgs.push(bad_value_error(m, "media", &el, tag.line, tag.col));
            }
        }

        // MIME type on link, embed, source, object
        if let Some(t) = tag.get_attr("type") {
            if matches!(el.as_str(), "link" | "embed" | "source" | "object")
                && !is_valid_mime_type(t)
            {
                msgs.push(bad_value_error(t, "type", &el, tag.line, tag.col));
            }
        }

        // 6. Language tag redundant script warning
        if let Some(lang) = tag.get_attr("lang") {
            if lang.eq_ignore_ascii_case("ja-Jpan") {
                msgs.push(bad_value_warning(lang, "lang", &el, tag.line, tag.col));
            }
        }

        // 7. URL attributes on elements
        validate_url_attributes(tag, &el, &mut msgs);

        // 8. Element-specific datatype rules (non-input elements)
        validate_element_datatypes(tag, &el, &mut msgs);

        msgs
    }
}
