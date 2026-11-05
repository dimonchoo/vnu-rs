use crate::context::{DocumentMode, ElementContext, ValidationContext};
use crate::engine::html_stack::is_void_element;
use crate::rule::Rule;
use vnu_core::VNuMessage;

#[inline]
pub fn is_foreign_or_xhtml(ctx: &ValidationContext, name_str: &str) -> bool {
    ctx.mode == DocumentMode::Svg
        || ctx.is_xhtml
        || name_str.eq_ignore_ascii_case("svg")
        || name_str.eq_ignore_ascii_case("math")
        || ctx.is_descendant_of("svg")
        || ctx.is_descendant_of("math")
}

pub fn handle_self_closing_and_stack(
    ctx: &mut ValidationContext,
    rules: &mut [Box<dyn Rule>],
    elem: ElementContext,
    self_closing: bool,
    line: usize,
    col: usize,
    messages: &mut Vec<VNuMessage>,
) {
    let name_str = elem.name.clone();
    let in_foreign = is_foreign_or_xhtml(ctx, &name_str);
    let is_void = is_void_element(&name_str);

    if !in_foreign && self_closing && !is_void {
        messages.push(VNuMessage::error(
            "Self-closing syntax (“/>”) used on a non-void HTML element. Ignoring the slash and treating as a start tag.",
            line,
            col,
        ));
        ctx.open_elements.push(elem);
    } else if in_foreign && self_closing && !is_void {
        ctx.open_elements.push(elem);
        for rule in rules {
            messages.extend(rule.on_end_tag(ctx, &name_str, line, col));
        }
        ctx.open_elements.pop();
    } else if !is_void && !self_closing {
        ctx.open_elements.push(elem);
    } else if !in_foreign && self_closing && is_void {
        messages.push(VNuMessage::info(
            "Trailing slash on void elements has no effect and interacts badly with unquoted attribute values.",
            line,
            col,
        ));
    }
}

pub fn handle_end_tag(
    ctx: &mut ValidationContext,
    rules: &mut [Box<dyn Rule>],
    stray_rule_indices: &[usize],
    name_str: &str,
    line: usize,
    col: usize,
    messages: &mut Vec<VNuMessage>,
) {
    if is_void_element(name_str) {
        if ctx.is_xhtml {
            return;
        }
        let mut handled = false;
        for &idx in stray_rule_indices {
            let stray = rules[idx].on_stray_end_tag(ctx, name_str, line, col);
            if !stray.is_empty() {
                messages.extend(stray);
                handled = true;
                break;
            }
        }
        if !handled {
            messages.push(VNuMessage::error(
                format!("Stray end tag “{}”.", name_str),
                line,
                col,
            ));
        }
        return;
    }

    // Implicit closure before closing parent containers
    if let Some(m) = crate::engine::html_stack::apply_implicit_closures(
        &mut ctx.open_elements,
        name_str,
        false,
        line,
        col,
    ) {
        messages.push(m);
    }

    if let Some(pos) = ctx
        .open_elements
        .iter()
        .rposition(|e| e.name.eq_ignore_ascii_case(name_str))
    {
        if pos + 1 < ctx.open_elements.len() {
            let current_open = ctx
                .open_elements
                .last()
                .map(|e| e.name.as_str())
                .unwrap_or("");
            messages.push(VNuMessage::error(
                format!(
                    "End tag “{}” did not match the name of the current open element (“{}”).",
                    name_str, current_open
                ),
                line,
                col,
            ));
        }

        for rule in rules {
            messages.extend(rule.on_end_tag(ctx, name_str, line, col));
        }
        ctx.open_elements.truncate(pos);
    } else if name_str.eq_ignore_ascii_case("p") {
        messages.push(VNuMessage::error(
            "No “p” element in scope but a “p” end tag seen.".to_string(),
            line,
            col,
        ));
    } else {
        let mut handled = false;
        for &idx in stray_rule_indices {
            let stray = rules[idx].on_stray_end_tag(ctx, name_str, line, col);
            if !stray.is_empty() {
                messages.extend(stray);
                handled = true;
                break;
            }
        }
        if !handled {
            messages.push(VNuMessage::error(
                format!("Stray end tag “{}”.", name_str),
                line,
                col,
            ));
        }
    }
}
