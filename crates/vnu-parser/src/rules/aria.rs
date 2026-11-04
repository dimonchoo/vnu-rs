use vnu_core::VNuMessage;
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;

const VALID_ARIA_ROLES: &[&str] = &[
    "alert", "alertdialog", "application", "article", "banner", "button", "cell", "checkbox",
    "columnheader", "combobox", "command", "complementary", "composite", "contentinfo",
    "definition", "dialog", "directory", "document", "feed", "figure", "form", "grid",
    "gridcell", "group", "heading", "img", "input", "landmark", "link", "list", "listbox",
    "listitem", "log", "main", "marquee", "math", "menu", "menubar", "menuitem",
    "menuitemcheckbox", "menuitemradio", "meter", "navigation", "none", "note", "option",
    "presentation", "progressbar", "radio", "radiogroup", "range", "region", "roletype", "row",
    "rowgroup", "rowheader", "scrollbar", "search", "searchbox", "section", "sectionhead",
    "select", "separator", "slider", "spinbutton", "status", "structure", "switch", "tab",
    "table", "tablist", "tabpanel", "term", "textbox", "timer", "toolbar", "tooltip", "tree",
    "treegrid", "treeitem", "widget", "window",
];

const VALID_ARIA_ATTRIBUTES: &[&str] = &[
    "aria-activedescendant", "aria-atomic", "aria-autocomplete", "aria-busy", "aria-checked",
    "aria-colcount", "aria-colindex", "aria-colspan", "aria-controls", "aria-current",
    "aria-describedby", "aria-details", "aria-disabled", "aria-dropeffect", "aria-errormessage",
    "aria-expanded", "aria-flowto", "aria-grabbed", "aria-haspopup", "aria-hidden",
    "aria-invalid", "aria-keyshortcuts", "aria-label", "aria-labelledby", "aria-level",
    "aria-live", "aria-modal", "aria-multiline", "aria-multiselectable", "aria-orientation",
    "aria-owns", "aria-placeholder", "aria-posinset", "aria-pressed", "aria-readonly",
    "aria-relevant", "aria-required", "aria-roledescription", "aria-rowcount", "aria-rowindex",
    "aria-rowspan", "aria-selected", "aria-setsize", "aria-sort", "aria-valuemax",
    "aria-valuemin", "aria-valuenow", "aria-valuetext",
];

#[derive(Default)]
pub struct AriaRule;

impl Rule for AriaRule {
    fn name(&self) -> &'static str {
        "aria"
    }

    fn on_start_tag(&mut self, _ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let elem_name = &tag.name;

        // Check role attribute
        if let Some(role_attr) = tag.get_attr("role") {
            for token in role_attr.split_whitespace() {
                let token_lower = token.to_ascii_lowercase();
                if !VALID_ARIA_ROLES.contains(&token_lower.as_str()) {
                    msgs.push(VNuMessage::error(
                        format!(
                            "Bad value “{}” for attribute “role” on element “{}”.",
                            token, elem_name
                        ),
                        tag.line,
                        tag.col,
                    ));
                }
            }
        }

        // Check aria-* attributes
        for (attr_name, _) in &tag.attrs {
            let attr_lower = attr_name.to_ascii_lowercase();
            if attr_lower.starts_with("aria-") && !VALID_ARIA_ATTRIBUTES.contains(&attr_lower.as_str()) {
                msgs.push(VNuMessage::error(
                    format!(
                        "Attribute “{}” not allowed on element “{}” at this point.",
                        attr_lower, elem_name
                    ),
                    tag.line,
                    tag.col,
                ));
            }
        }

        msgs
    }
}
