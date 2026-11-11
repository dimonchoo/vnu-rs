use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use vnu_core::VNuMessage;

const MATH_CHILDREN_LIST: &str = "“abs”, “and”, “apply”, “approx”, “arccos”, “arccosh”, “arccot”, “arccoth”, “arccsc”, “arccsch”, “arcsec”, “arcsech”, “arcsin”, “arcsinh”, “arctan”, “arctanh”, “arg”, “bind”, “card”, “cartesianproduct”, “cbytes”, “ceiling”, “cerror”, “ci”, “cn”, “codomain”, “complexes”, “compose”, “conjugate”, “cos”, “cosh”, “cot”, “coth”, “cs”, “csc”, “csch”, “csymbol”, “curl”, “declare”, “determinant”, “diff”, “divergence”, “divide”, “domain”, “emptyset”, “eq”, “equivalent”, “eulergamma”, “exists”, “exp”, “exponentiale”, “factorial”, “factorof”, “false”, “floor”, “fn”, “forall”, “gcd”, “geq”, “grad”, “gt”, “ident”, “image”, “imaginary”, “imaginaryi”, “implies”, “in”, “infinity”, “int”, “integers”, “intersect”, “interval”, “inverse”, “lambda”, “laplacian”, “lcm”, “leq”, “limit”, “list”, “ln”, “log”, “lt”, “maction”, “maligngroup”, “malignmark”, “matrix”, “matrixrow”, “max”, “mean”, “median”, “menclose”, “merror”, “mfenced”, “mfrac”, “mi”, “min”, “minus”, “mlongdiv”, “mmultiscripts”, “mn”, “mo”, “mode”, “moment”, “mover”, “mpadded”, “mphantom”, “mroot”, “mrow”, “ms”, “mspace”, “msqrt”, “mstack”, “mstyle”, “msub”, “msubsup”, “msup”, “mtable”, “mtext”, “munder”, “munderover”, “naturalnumbers”, “neq”, “not”, “notanumber”, “notin”, “notprsubset”, “notsubset”, “or”, “outerproduct”, “partialdiff”, “pi”, “piecewise”, “plus”, “power”, “primes”, “product”, “prsubset”, “quotient”, “rationals”, “real”, “reals”, “reln”, “rem”, “root”, “scalarproduct”, “sdev”, “sec”, “sech”, “selector”, “semantics”, “set”, “setdiff”, “share”, “sin”, “sinh”, “subset”, “sum”, “tan”, “tanh”, “tendsto”, “times”, “transpose”, “true”, “union”, “variance”, “vector”, “vectorproduct”, “xor”";

const KNOWN_MATHML_ELEMENTS: &[&str] = &[
    "math",
    "maction",
    "annotation",
    "annotation-xml",
    "maligngroup",
    "malignmark",
    "menclose",
    "merror",
    "mfenced",
    "mfrac",
    "mi",
    "mmultiscripts",
    "mn",
    "mo",
    "mover",
    "mpadded",
    "mphantom",
    "mroot",
    "mrow",
    "ms",
    "mspace",
    "msqrt",
    "mstyle",
    "msub",
    "msubsup",
    "msup",
    "mtable",
    "mtd",
    "mtext",
    "mtr",
    "munder",
    "munderover",
    "semantics",
    "mprescripts",
    "none",
];

const TWO_CHILDREN_MATH_ELEMENTS: &[&str] = &["mfrac", "mover", "mroot", "msub", "msup", "munder"];

const THREE_CHILDREN_MATH_ELEMENTS: &[&str] = &["msubsup", "munderover"];

#[derive(Default)]
pub struct ForeignMathRule {
    math_depth: usize,
    math_child_counts: Vec<(String, usize)>,
}

impl Rule for ForeignMathRule {
    fn name(&self) -> &'static str {
        "foreign-math"
    }

    fn reset(&mut self) {
        self.math_depth = 0;
        self.math_child_counts.clear();
    }

    fn interested_in_start_tag(&self, tag: &ElementContext) -> bool {
        self.math_depth > 0 || tag.name.eq_ignore_ascii_case("math")
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = tag.name.to_ascii_lowercase();

        if name_lower == "math" {
            self.math_depth += 1;
        }

        if let Some((_, count)) = self.math_child_counts.last_mut() {
            *count += 1;
        }

        if TWO_CHILDREN_MATH_ELEMENTS.contains(&name_lower.as_str())
            || THREE_CHILDREN_MATH_ELEMENTS.contains(&name_lower.as_str())
        {
            self.math_child_counts.push((name_lower.clone(), 0));
        }

        if ctx.is_descendant_of("math") {
            let parent_name = ctx
                .parent()
                .map(|p| p.name.to_ascii_lowercase())
                .unwrap_or_default();

            // Foreign HTML element inside math
            let is_html_tag = matches!(
                name_lower.as_str(),
                "div" | "p" | "span" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "a" | "table"
            );
            // Unknown element
            if !KNOWN_MATHML_ELEMENTS.contains(&name_lower.as_str()) && !is_html_tag {
                msgs.push(VNuMessage::error(
                    format!(
                        "Element “{}” not allowed as child of “{}” in this context.",
                        name_lower, parent_name
                    ),
                    tag.line,
                    tag.col,
                ));
            }

            // Hierarchical constraints
            if name_lower == "annotation" && parent_name != "semantics" {
                msgs.push(VNuMessage::error(
                    format!(
                        "Element “annotation” not allowed as child of “{}” in this context.",
                        parent_name
                    ),
                    tag.line,
                    tag.col,
                ));
            }
            if name_lower == "mprescripts" && parent_name != "mmultiscripts" {
                msgs.push(VNuMessage::error(
                    format!(
                        "Element “mprescripts” not allowed as child of “{}” in this context.",
                        parent_name
                    ),
                    tag.line,
                    tag.col,
                ));
            }
            if name_lower == "mtd" && parent_name != "mtr" {
                msgs.push(VNuMessage::error(
                    format!(
                        "Element “mtd” not allowed as child of “{}” in this context.",
                        parent_name
                    ),
                    tag.line,
                    tag.col,
                ));
            }
            if name_lower == "mtr" && parent_name != "mtable" {
                msgs.push(VNuMessage::error(
                    format!(
                        "Element “mtr” not allowed as child of “{}” in this context.",
                        parent_name
                    ),
                    tag.line,
                    tag.col,
                ));
            }
        }

        if name_lower == "math" {
            if let Some(disp) = tag.get_attr("display") {
                if disp != "block" && disp != "inline" {
                    msgs.push(VNuMessage::error(
                        format!(
                            "Bad value “{}” for attribute “display” on element “math”.",
                            disp
                        ),
                        tag.line,
                        tag.col,
                    ));
                }
            }
            if let Some(ov) = tag.get_attr("overflow") {
                if !matches!(ov, "linebreak" | "scroll" | "elide" | "truncate" | "scale") {
                    msgs.push(VNuMessage::error(
                        format!(
                            "Bad value “{}” for attribute “overflow” on element “math”.",
                            ov
                        ),
                        tag.line,
                        tag.col,
                    ));
                }
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
        let name_lower = name.to_ascii_lowercase();

        if name_lower == "math" && self.math_depth > 0 {
            self.math_depth -= 1;
        }

        if let Some((elem, count)) = self.math_child_counts.last() {
            if elem == &name_lower {
                let required = if THREE_CHILDREN_MATH_ELEMENTS.contains(&elem.as_str()) {
                    3
                } else {
                    2
                };
                if *count < required {
                    msgs.push(VNuMessage::error(
                        format!(
                            "Element “{}” is missing a required instance of one or more of the following child elements: {}.",
                            elem, MATH_CHILDREN_LIST
                        ),
                        line,
                        col,
                    ));
                }
                self.math_child_counts.pop();
            }
        }

        msgs
    }
}
