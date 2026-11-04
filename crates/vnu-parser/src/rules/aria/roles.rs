pub const VALID_ARIA_ROLES: &[&str] = &[
    "alert", "alertdialog", "application", "article", "banner", "blockquote", "button",
    "caption", "cell", "checkbox", "code", "columnheader", "combobox", "complementary",
    "contentinfo", "definition", "deletion", "dialog", "doc-abstract", "doc-acknowledgments",
    "doc-afterword", "doc-appendix", "doc-backlink", "doc-biblioentry", "doc-bibliography",
    "doc-biblioref", "doc-chapter", "doc-colophon", "doc-conclusion", "doc-cover", "doc-credit",
    "doc-credits", "doc-dedication", "doc-endnote", "doc-endnotes", "doc-epigraph", "doc-epilogue",
    "doc-errata", "doc-example", "doc-footnote", "doc-foreword", "doc-glossary", "doc-glossref",
    "doc-index", "doc-introduction", "doc-noteref", "doc-notice", "doc-pagebreak", "doc-pagelist",
    "doc-part", "doc-preface", "doc-prologue", "doc-pullquote", "doc-qna", "doc-subtitle",
    "doc-tip", "doc-toc", "document", "emphasis", "feed", "figure", "form", "graphics-document",
    "graphics-object", "graphics-symbol", "grid", "gridcell", "group", "heading", "image",
    "img", "insertion", "link", "list", "listbox", "listitem", "log", "main", "marquee", "math",
    "menu", "menubar", "menuitem", "menuitemcheckbox", "menuitemradio", "meter", "navigation",
    "none", "note", "option", "paragraph", "presentation", "progressbar", "radio", "radiogroup",
    "region", "row", "rowgroup", "rowheader", "scrollbar", "search", "searchbox", "separator",
    "slider", "spinbutton", "status", "strong", "subscript", "superscript", "switch", "tab",
    "table", "tablist", "tabpanel", "term", "textbox", "time", "timer", "toolbar", "tooltip",
    "tree", "treegrid", "treeitem",
];

pub const VALID_ARIA_ATTRIBUTES: &[&str] = &[
    "aria-activedescendant", "aria-atomic", "aria-autocomplete", "aria-braillelabel",
    "aria-brailleroledescription", "aria-busy", "aria-checked", "aria-colcount", "aria-colindex",
    "aria-colspan", "aria-controls", "aria-current", "aria-description", "aria-describedby",
    "aria-details", "aria-disabled", "aria-dropeffect", "aria-errormessage", "aria-expanded",
    "aria-flowto", "aria-grabbed", "aria-haspopup", "aria-hidden", "aria-invalid",
    "aria-keyshortcuts", "aria-label", "aria-labelledby", "aria-level", "aria-live", "aria-modal",
    "aria-multiline", "aria-multiselectable", "aria-orientation", "aria-owns", "aria-placeholder",
    "aria-posinset", "aria-pressed", "aria-readonly", "aria-relevant", "aria-required",
    "aria-roledescription", "aria-rowcount", "aria-rowindex", "aria-rowspan", "aria-selected",
    "aria-setsize", "aria-sort", "aria-valuemax", "aria-valuemin", "aria-valuenow", "aria-valuetext",
];

pub const PROHIBIT_INTERACTIVE_ROLES: &[&str] = &[
    "checkbox", "menuitem", "menuitemcheckbox", "menuitemradio", "option", "radio", "switch", "tab",
];
