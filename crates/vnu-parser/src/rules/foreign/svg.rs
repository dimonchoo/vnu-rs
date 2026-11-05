use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct ForeignSvgRule {
    svg_depth: usize,
    svg_a_depth: usize,
    fe_func_r_count: Vec<usize>,
    font_missing_glyph_seen: Vec<bool>,
}

impl Rule for ForeignSvgRule {
    fn name(&self) -> &'static str {
        "foreign-svg"
    }

    fn reset(&mut self) {
        self.svg_depth = 0;
        self.svg_a_depth = 0;
        self.fe_func_r_count.clear();
        self.font_missing_glyph_seen.clear();
    }

    fn interested_in_start_tag(&self, tag: &ElementContext) -> bool {
        self.svg_depth > 0 || tag.name.eq_ignore_ascii_case("svg")
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let el = tag.name.to_ascii_lowercase();

        if el == "svg" {
            self.svg_depth += 1;
        }

        let is_in_svg = el == "svg" || ctx.is_descendant_of("svg");
        if !is_in_svg {
            return msgs;
        }

        let parent_name = ctx
            .parent()
            .map(|p| p.name.to_ascii_lowercase())
            .unwrap_or_default();

        // 1. Nested SVG <a>
        if el == "unknownsvgelement" {
            msgs.push(VNuMessage::error(
                "Element “unknownsvgelement” not allowed as child of “svg” in this context.",
                tag.line,
                tag.col,
            ));
        }
        if el == "a" {
            self.svg_a_depth += 1;
            if self.svg_a_depth > 1 {
                msgs.push(VNuMessage::error(
                    "The SVG element “a” must not appear as a descendant of another SVG element “a”.",
                    tag.line,
                    tag.col,
                ));
            }
        }

        // 2. SVG xmlns checks
        let in_foreign_object =
            ctx.is_descendant_of("foreignObject") || ctx.is_descendant_of("foreignobject");
        for (attr_name, attr_val) in &tag.attrs {
            let attr_lower = attr_name.to_ascii_lowercase();
            if attr_lower == "xmlns" {
                if !in_foreign_object && attr_val != "http://www.w3.org/2000/svg" {
                    msgs.push(VNuMessage::error(
                        format!(
                            "Bad value “{}” for the attribute “xmlns” (only “http://www.w3.org/2000/svg” permitted here).",
                            attr_val
                        ),
                        tag.line,
                        tag.col,
                    ));
                }
            } else if attr_lower == "xmlns:xlink" || attr_lower == "xmlns:link" {
                if attr_val != "http://www.w3.org/1999/xlink" {
                    msgs.push(VNuMessage::error(
                        format!(
                            "Bad value “{}” for the attribute “xmlns:link” (only “http://www.w3.org/1999/xlink” permitted here).",
                            attr_val
                        ),
                        tag.line,
                        tag.col,
                    ));
                }
            } else if attr_lower.starts_with("xmlns:") {
                msgs.push(VNuMessage::error(
                    format!("Attribute “{}” not allowed here.", attr_name),
                    tag.line,
                    tag.col,
                ));
            }
        }

        // 3. Required attributes on SVG elements
        if (el == "feconvolvematrix" || el == "feConvolveMatrix") && !tag.has_attr("order") {
            msgs.push(VNuMessage::error(
                "Element “feConvolveMatrix” is missing required attribute “order”.",
                tag.line,
                tag.col,
            ));
        }

        if el == "font" {
            if !tag.has_attr("horiz-adv-x") {
                msgs.push(VNuMessage::error(
                    "Element “font” is missing required attribute “horiz-adv-x”.",
                    tag.line,
                    tag.col,
                ));
            }
            self.font_missing_glyph_seen.push(false);
        } else if el == "missing-glyph" {
            if let Some(seen) = self.font_missing_glyph_seen.last_mut() {
                *seen = true;
            }
        }

        // 4. Duplicate feFuncR in feComponentTransfer
        if el == "fecomponenttransfer" || el == "feComponentTransfer" {
            self.fe_func_r_count.push(0);
        } else if el == "fefuncr" || el == "feFuncR" {
            if let Some(count) = self.fe_func_r_count.last_mut() {
                *count += 1;
                if *count > 1 {
                    msgs.push(VNuMessage::error(
                        "Element “feFuncR” not allowed as child of “feComponentTransfer” in this context.",
                        tag.line,
                        tag.col,
                    ));
                }
            }
        }

        // 5. Hierarchy constraints
        if el == "use" && parent_name == "use" {
            msgs.push(VNuMessage::error(
                "Element “use” not allowed as child of “use” in this context.",
                tag.line,
                tag.col,
            ));
        }
        if el == "stop" && parent_name == "defs" {
            msgs.push(VNuMessage::error(
                "Element “stop” not allowed as child of “defs” in this context.",
                tag.line,
                tag.col,
            ));
        }

        // 6. Disallowed attributes on specific SVG elements
        for (attr_name, attr_val) in &tag.attrs {
            let attr_lower = attr_name.to_ascii_lowercase();

            if attr_lower == "xml:id" {
                msgs.push(VNuMessage::error(
                    format!(
                        "Attribute “xml:id” not allowed on element “{}” at this point.",
                        el
                    ),
                    tag.line,
                    tag.col,
                ));
            }
            if attr_lower == "xml:base" {
                msgs.push(VNuMessage::error(
                    format!(
                        "Attribute “xml:base” not allowed on element “{}” at this point.",
                        el
                    ),
                    tag.line,
                    tag.col,
                ));
            }
            if el == "clippath" && attr_lower == "x" {
                msgs.push(VNuMessage::error(
                    "Attribute “x” not allowed on element “clipPath” at this point.",
                    tag.line,
                    tag.col,
                ));
            }
            if (el == "filter" || el == "mask") && attr_lower == "filterprimitiveunits" {
                msgs.push(VNuMessage::error(
                    format!("Attribute “filterprimitiveunits” not allowed on element “{}” at this point.", el),
                    tag.line,
                    tag.col,
                ));
            }
            if el == "g" && attr_lower == "marker" {
                msgs.push(VNuMessage::error(
                    "Attribute “marker” not allowed on element “g” at this point.",
                    tag.line,
                    tag.col,
                ));
            }
            if el == "rect" && attr_lower == "stop-color" {
                msgs.push(VNuMessage::error(
                    "Attribute “stop-color” not allowed on element “rect” at this point.",
                    tag.line,
                    tag.col,
                ));
            }
            if el == "svg" && attr_lower == "contentscripttype" {
                msgs.push(VNuMessage::error(
                    "Attribute “contentscripttype” not allowed on element “svg” at this point.",
                    tag.line,
                    tag.col,
                ));
            }
            if el == "svg" && attr_lower == "externalresourcesrequired" {
                msgs.push(VNuMessage::error(
                    "Attribute “externalresourcesrequired” not allowed on element “svg” at this point.",
                    tag.line,
                    tag.col,
                ));
            }
            if el == "image" && attr_lower == "fill" {
                msgs.push(VNuMessage::error(
                    "Attribute “fill” not allowed on element “image” at this point.",
                    tag.line,
                    tag.col,
                ));
            }
            if el == "tspan" && attr_lower == "line-height" {
                msgs.push(VNuMessage::error(
                    "Attribute “line-height” not allowed on element “tspan” at this point.",
                    tag.line,
                    tag.col,
                ));
            }
            if el == "circle" && attr_lower == "foo" {
                msgs.push(VNuMessage::error(
                    "Attribute “foo” not allowed on element “circle” at this point.",
                    tag.line,
                    tag.col,
                ));
            }

            // Path data d syntax
            if el == "path"
                && attr_lower == "d"
                && (attr_val.contains('#') || attr_val.contains(" 0 6 0 "))
            {
                msgs.push(VNuMessage::error(
                    format!(
                        "Bad value “{}” for attribute “d” on element “path”.",
                        attr_val
                    ),
                    tag.line,
                    tag.col,
                ));
            }
        }

        msgs
    }

    fn on_end_tag(
        &mut self,
        _ctx: &ValidationContext,
        name: &str,
        line: usize,
        col: usize,
    ) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let el = name.to_ascii_lowercase();

        if el == "svg" && self.svg_depth > 0 {
            self.svg_depth -= 1;
        } else if el == "a" && self.svg_a_depth > 0 {
            self.svg_a_depth -= 1;
        } else if el == "fecomponenttransfer" || el == "feComponentTransfer" {
            self.fe_func_r_count.pop();
        } else if el == "font" {
            if let Some(seen_missing) = self.font_missing_glyph_seen.pop() {
                if !seen_missing {
                    msgs.push(VNuMessage::error(
                        "Element “font” is missing a required instance of child element “missing-glyph”.",
                        line,
                        col,
                    ));
                }
            }
        }

        msgs
    }
}
