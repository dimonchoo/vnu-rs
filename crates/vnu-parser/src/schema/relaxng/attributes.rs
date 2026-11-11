//! What happens to attributes between nu htmlparser and Jing: SAX naming
//! (`AttributeName`), `AttributesPermutingXMLReaderWrapper`, and the attribute
//! filters of the schema wrappers.

use super::{MATHML_NS, SVG_NS, XHTML_NS};
use crate::treebuilder::Ns;
use vnu_core::VNuMessage;
use vnu_relaxng::Attribute;

const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";
const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

/// An attribute as nu htmlparser reports it to SAX: HTML elements get `lang` as
/// `xml:lang` (`setMappingLangToXmlLang(true)`); SVG and MathML elements get their
/// adjusted names and the `xlink:`/`xml:` namespaces; `xmlns` attributes are never
/// reported.
pub(super) fn sax_attribute<'a>(ns: Ns, name: &'a str, value: &'a str) -> Option<Attribute<'a>> {
    // `AttributeName.createAttributeName` flags every `xmlns:*` name IS_XMLNS.
    if name == "xmlns" || name.starts_with("xmlns:") {
        return None;
    }
    let (namespace_uri, local_name) = match ns {
        Ns::Html => match name {
            "lang" => (XML_NS, "lang"),
            _ => ("", name),
        },
        Ns::Svg | Ns::MathMl => match name {
            "xml:lang" => (XML_NS, "lang"),
            "xml:space" => (XML_NS, "space"),
            _ => match name.strip_prefix("xlink:") {
                Some(
                    local @ ("href" | "title" | "role" | "arcrole" | "type" | "show" | "actuate"),
                ) => (XLINK_NS, local),
                _ if ns == Ns::Svg => ("", adjust_svg_attribute(name)),
                _ if name == "definitionurl" => ("", "definitionURL"),
                _ => ("", name),
            },
        },
    };
    Some(Attribute {
        namespace_uri,
        local_name,
        value,
    })
}

/// The SVG-mode local names of nu htmlparser's `AttributeName`.
fn adjust_svg_attribute(name: &str) -> &str {
    const SVG: &[(&str, &str)] = &[
        ("attributename", "attributeName"),
        ("attributetype", "attributeType"),
        ("basefrequency", "baseFrequency"),
        ("baseprofile", "baseProfile"),
        ("calcmode", "calcMode"),
        ("clippathunits", "clipPathUnits"),
        ("diffuseconstant", "diffuseConstant"),
        ("edgemode", "edgeMode"),
        ("filterunits", "filterUnits"),
        ("glyphref", "glyphRef"),
        ("gradienttransform", "gradientTransform"),
        ("gradientunits", "gradientUnits"),
        ("kernelmatrix", "kernelMatrix"),
        ("kernelunitlength", "kernelUnitLength"),
        ("keypoints", "keyPoints"),
        ("keysplines", "keySplines"),
        ("keytimes", "keyTimes"),
        ("lengthadjust", "lengthAdjust"),
        ("limitingconeangle", "limitingConeAngle"),
        ("markerheight", "markerHeight"),
        ("markerunits", "markerUnits"),
        ("markerwidth", "markerWidth"),
        ("maskcontentunits", "maskContentUnits"),
        ("maskunits", "maskUnits"),
        ("numoctaves", "numOctaves"),
        ("pathlength", "pathLength"),
        ("patterncontentunits", "patternContentUnits"),
        ("patterntransform", "patternTransform"),
        ("patternunits", "patternUnits"),
        ("pointsatx", "pointsAtX"),
        ("pointsaty", "pointsAtY"),
        ("pointsatz", "pointsAtZ"),
        ("preservealpha", "preserveAlpha"),
        ("preserveaspectratio", "preserveAspectRatio"),
        ("primitiveunits", "primitiveUnits"),
        ("refx", "refX"),
        ("refy", "refY"),
        ("repeatcount", "repeatCount"),
        ("repeatdur", "repeatDur"),
        ("requiredextensions", "requiredExtensions"),
        ("requiredfeatures", "requiredFeatures"),
        ("specularconstant", "specularConstant"),
        ("specularexponent", "specularExponent"),
        ("spreadmethod", "spreadMethod"),
        ("startoffset", "startOffset"),
        ("stddeviation", "stdDeviation"),
        ("stitchtiles", "stitchTiles"),
        ("surfacescale", "surfaceScale"),
        ("systemlanguage", "systemLanguage"),
        ("tablevalues", "tableValues"),
        ("targetx", "targetX"),
        ("targety", "targetY"),
        ("textlength", "textLength"),
        ("viewbox", "viewBox"),
        ("viewtarget", "viewTarget"),
        ("xchannelselector", "xChannelSelector"),
        ("ychannelselector", "yChannelSelector"),
        ("zoomandpan", "zoomAndPan"),
    ];
    match SVG.binary_search_by(|(k, _)| k.cmp(&name)) {
        Ok(i) => SVG[i].1,
        Err(_) => name,
    }
}

/// `AttributesPermutingXMLReaderWrapper.startElement`.
pub(super) fn permute_attributes(ns: Ns, name: &str, atts: &mut [Attribute<'_>]) {
    if ns != Ns::Html {
        return;
    }
    let index = |atts: &[Attribute<'_>], local: &str| {
        atts.iter()
            .position(|a| a.namespace_uri.is_empty() && a.local_name == local)
    };
    let pull_up = |atts: &mut [Attribute<'_>], local: &str| {
        if let Some(i) = index(atts, local).filter(|&i| i > 0) {
            atts[..=i].rotate_right(1);
        }
    };
    let push_down = |atts: &mut [Attribute<'_>], local: &str| {
        if let Some(i) = index(atts, local).filter(|&i| i + 1 < atts.len()) {
            atts[i..].rotate_left(1);
        }
    };
    let after_first =
        |atts: &[Attribute<'_>], local: &str| index(atts, local).is_some_and(|i| i > 0);
    if (name == "input" || name == "command") && after_first(atts, "type") {
        pull_up(atts, "role");
        pull_up(atts, "type");
        pull_up(atts, "template");
    } else if name == "textarea" && after_first(atts, "wrap") {
        pull_up(atts, "wrap");
        pull_up(atts, "template");
    } else if name == "script" && after_first(atts, "src") {
        pull_up(atts, "src");
        pull_up(atts, "template");
    } else if name == "meta"
        && index(atts, "content").map_or(-1, |i| i as isize) < atts.len() as isize - 1
    {
        push_down(atts, "content");
        pull_up(atts, "template");
    } else if after_first(atts, "template") || after_first(atts, "role") {
        pull_up(atts, "role");
        pull_up(atts, "template");
    }
}

/// The attribute filters of the schema wrappers, outermost first, with the
/// messages the role filter reports itself.
pub(super) fn filter_attributes(
    uri: &str,
    atts: &mut Vec<Attribute<'_>>,
    line: usize,
    col: usize,
    out: &mut Vec<VNuMessage>,
) {
    // RoleAttributeFilteringContentHandlerWrapper
    if uri == XHTML_NS || uri == SVG_NS {
        for att in atts.iter_mut() {
            if att.namespace_uri.is_empty() && att.local_name == "role" {
                att.value = first_matching_aria_role(att.value.trim(), line, col, out);
            }
        }
    }
    // XmlLangAttributeDroppingContentHandlerWrapper
    if uri == XHTML_NS {
        atts.retain(|a| !(a.namespace_uri.is_empty() && a.local_name == "xml:lang"));
    }
    if uri == XHTML_NS || uri == SVG_NS || uri == MATHML_NS {
        // AriaAttributeDroppingContentHandlerWrapper drops known aria-* attributes
        // whose value `== ""`. That compares references: only the interned "" of an
        // attribute written without a value matches, never a parsed `=""`. The tree
        // events do not say which form was used, so nothing is dropped here.
        // DataAttributeDroppingContentHandlerWrapper
        atts.retain(|a| {
            !(a.namespace_uri.is_empty()
                && a.local_name.len() > 5
                && a.local_name.starts_with("data-"))
        });
    }
}

/// `RoleAttributeFilteringContentHandlerWrapper.getFirstMatchingAriaRoleFromTokenList`:
/// the first non-abstract ARIA role among the tokens, or all of them when there is
/// none. Unrecognized tokens next to a role are an error, further roles an info.
fn first_matching_aria_role<'a>(
    token_list: &'a str,
    line: usize,
    col: usize,
    out: &mut Vec<VNuMessage>,
) -> &'a str {
    if token_list.is_empty() {
        return "";
    }
    let mut role_value = None;
    let mut unrecognized: Vec<&str> = Vec::new();
    let mut superfluous: Vec<&str> = Vec::new();
    for token in token_list
        .split([' ', '\t', '\n', '\r'])
        .filter(|t| !t.is_empty())
    {
        if NON_ABSTRACT_ARIA_ROLES.binary_search(&token).is_err() {
            unrecognized.push(token);
        } else if role_value.is_none() {
            role_value = Some(token);
        } else if token != "presentation" {
            superfluous.push(token);
        }
    }
    if role_value.is_some() && !unrecognized.is_empty() {
        out.push(VNuMessage::error(
            format!(
                "Discarding unrecognized{} from value of attribute “role”. Browsers ignore any token that is not a defined ARIA non-abstract role.",
                render_token_list(&unrecognized)
            ),
            line,
            col,
        ));
    }
    if role_value.is_some() && !superfluous.is_empty() {
        out.push(VNuMessage::info(
            format!(
                "Discarding superfluous{} from value of attribute “role”. Browsers only process the first token found that is a defined ARIA non-abstract role.",
                render_token_list(&superfluous)
            ),
            line,
            col,
        ));
    }
    role_value.unwrap_or(token_list)
}

/// `RoleAttributeFilteringContentHandlerWrapper.renderTokenList`.
fn render_token_list(tokens: &[&str]) -> String {
    let mut sb = String::from(if tokens.len() > 1 {
        " tokens "
    } else {
        " token "
    });
    for (i, token) in tokens.iter().enumerate() {
        if i > 0 {
            sb.push_str(", ");
        }
        sb.push('“');
        sb.push_str(token);
        sb.push('”');
    }
    sb
}

/// `RoleAttributeFilteringContentHandlerWrapper.NON_ABSTRACT_ARIA_ROLES` (sorted).
const NON_ABSTRACT_ARIA_ROLES: &[&str] = &include!("aria_roles.in");

#[cfg(test)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_tables_are_sorted() {
        assert!(NON_ABSTRACT_ARIA_ROLES.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(adjust_svg_attribute("viewbox"), "viewBox");
        assert_eq!(adjust_svg_attribute("zoomandpan"), "zoomAndPan");
        assert_eq!(adjust_svg_attribute("attributename"), "attributeName");
        assert_eq!(adjust_svg_attribute("fill"), "fill");
    }
}
