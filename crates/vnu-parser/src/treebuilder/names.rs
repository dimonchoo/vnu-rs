//! Element names, dispatch groups and flags of nu htmlparser
//! (`htmlparser/src/nu/validator/htmlparser/impl/ElementName.java`).

pub(super) mod groups {
    pub const OTHER: u32 = 0;
    pub const A: u32 = 1;
    pub const BASE: u32 = 2;
    pub const BODY: u32 = 3;
    pub const BR: u32 = 4;
    pub const BUTTON: u32 = 5;
    pub const CAPTION: u32 = 6;
    pub const COL: u32 = 7;
    pub const COLGROUP: u32 = 8;
    pub const FORM: u32 = 9;
    pub const FRAME: u32 = 10;
    pub const FRAMESET: u32 = 11;
    pub const IMAGE: u32 = 12;
    pub const INPUT: u32 = 13;
    pub const RT_OR_RP: u32 = 14;
    pub const LI: u32 = 15;
    pub const LINK_OR_BASEFONT_OR_BGSOUND: u32 = 16;
    pub const MATH: u32 = 17;
    pub const META: u32 = 18;
    pub const SVG: u32 = 19;
    pub const HEAD: u32 = 20;
    pub const HR: u32 = 22;
    pub const HTML: u32 = 23;
    pub const NOBR: u32 = 24;
    pub const NOFRAMES: u32 = 25;
    pub const NOSCRIPT: u32 = 26;
    pub const OPTGROUP: u32 = 27;
    pub const OPTION: u32 = 28;
    pub const P: u32 = 29;
    pub const PLAINTEXT: u32 = 30;
    pub const SCRIPT: u32 = 31;
    pub const SELECT: u32 = 32;
    pub const STYLE: u32 = 33;
    pub const TABLE: u32 = 34;
    pub const TEXTAREA: u32 = 35;
    pub const TITLE: u32 = 36;
    pub const TR: u32 = 37;
    pub const XMP: u32 = 38;
    pub const TBODY_OR_THEAD_OR_TFOOT: u32 = 39;
    pub const TD_OR_TH: u32 = 40;
    pub const DD_OR_DT: u32 = 41;
    pub const H1_OR_H2_OR_H3_OR_H4_OR_H5_OR_H6: u32 = 42;
    pub const MARQUEE_OR_APPLET: u32 = 43;
    pub const PRE_OR_LISTING: u32 = 44;
    pub const B_OR_BIG_OR_CODE_OR_EM_OR_I_OR_S_OR_SMALL_OR_STRIKE_OR_STRONG_OR_TT_OR_U: u32 = 45;
    pub const UL_OR_OL_OR_DL: u32 = 46;
    pub const IFRAME: u32 = 47;
    pub const EMBED: u32 = 48;
    pub const AREA_OR_WBR: u32 = 49;
    pub const DIV_OR_BLOCKQUOTE_OR_CENTER_OR_MENU: u32 = 50;
    pub const ADDRESS_OR_ARTICLE_OR_ASIDE_OR_DETAILS_OR_DIALOG_OR_DIR_OR_FIGCAPTION_OR_FIGURE_OR_FOOTER_OR_HEADER_OR_HGROUP_OR_MAIN_OR_NAV_OR_SEARCH_OR_SECTION_OR_SUMMARY: u32 = 51;
    pub const RUBY_OR_SPAN_OR_SUB_OR_SUP_OR_VAR: u32 = 52;
    pub const RB_OR_RTC: u32 = 53;
    pub const PARAM_OR_SOURCE_OR_TRACK: u32 = 55;
    pub const MGLYPH_OR_MALIGNMARK: u32 = 56;
    pub const MI_MO_MN_MS_MTEXT: u32 = 57;
    pub const ANNOTATION_XML: u32 = 58;
    pub const FOREIGNOBJECT_OR_DESC: u32 = 59;
    pub const NOEMBED: u32 = 60;
    pub const FIELDSET: u32 = 61;
    pub const OUTPUT: u32 = 62;
    pub const OBJECT: u32 = 63;
    pub const FONT: u32 = 64;
    pub const KEYGEN: u32 = 65;
    pub const TEMPLATE: u32 = 66;
    pub const IMG: u32 = 67;

    pub const GROUP_MASK: u32 = 127;
    pub const NOT_INTERNED: u32 = 1 << 30;
    pub const SPECIAL: u32 = 1 << 29;
    pub const FOSTER_PARENTING: u32 = 1 << 28;
    pub const SCOPING: u32 = 1 << 27;
    pub const SCOPING_AS_SVG: u32 = 1 << 26;
    pub const SCOPING_AS_MATHML: u32 = 1 << 25;
    pub const HTML_INTEGRATION_POINT: u32 = 1 << 24;
    pub const OPTIONAL_END_TAG: u32 = 1 << 23;
}

// Generated from ElementName.java. Do not edit by hand.
use groups::*;

/// (local name, SVG camel-case name, group and flags)
pub(super) static ELEMENT_NAMES: &[(&str, &str, u32)] = &[
    ("a", "a", A),
    ("abbr", "abbr", OTHER),
    ("acronym", "acronym", OTHER),
    ("address", "address", ADDRESS_OR_ARTICLE_OR_ASIDE_OR_DETAILS_OR_DIALOG_OR_DIR_OR_FIGCAPTION_OR_FIGURE_OR_FOOTER_OR_HEADER_OR_HGROUP_OR_MAIN_OR_NAV_OR_SEARCH_OR_SECTION_OR_SUMMARY | SPECIAL),
    ("altglyph", "altGlyph", OTHER),
    ("altglyphdef", "altGlyphDef", OTHER),
    ("altglyphitem", "altGlyphItem", OTHER),
    ("animate", "animate", OTHER),
    ("animatecolor", "animateColor", OTHER),
    ("animatemotion", "animateMotion", OTHER),
    ("animatetransform", "animateTransform", OTHER),
    ("annotation-xml", "annotation-xml", ANNOTATION_XML | SCOPING_AS_MATHML),
    ("applet", "applet", MARQUEE_OR_APPLET | SPECIAL | SCOPING),
    ("area", "area", AREA_OR_WBR | SPECIAL),
    ("article", "article", ADDRESS_OR_ARTICLE_OR_ASIDE_OR_DETAILS_OR_DIALOG_OR_DIR_OR_FIGCAPTION_OR_FIGURE_OR_FOOTER_OR_HEADER_OR_HGROUP_OR_MAIN_OR_NAV_OR_SEARCH_OR_SECTION_OR_SUMMARY | SPECIAL),
    ("aside", "aside", ADDRESS_OR_ARTICLE_OR_ASIDE_OR_DETAILS_OR_DIALOG_OR_DIR_OR_FIGCAPTION_OR_FIGURE_OR_FOOTER_OR_HEADER_OR_HGROUP_OR_MAIN_OR_NAV_OR_SEARCH_OR_SECTION_OR_SUMMARY | SPECIAL),
    ("audio", "audio", OTHER),
    ("b", "b", B_OR_BIG_OR_CODE_OR_EM_OR_I_OR_S_OR_SMALL_OR_STRIKE_OR_STRONG_OR_TT_OR_U),
    ("base", "base", BASE | SPECIAL),
    ("basefont", "basefont", LINK_OR_BASEFONT_OR_BGSOUND | SPECIAL),
    ("bdi", "bdi", OTHER),
    ("bdo", "bdo", OTHER),
    ("bgsound", "bgsound", LINK_OR_BASEFONT_OR_BGSOUND | SPECIAL),
    ("big", "big", B_OR_BIG_OR_CODE_OR_EM_OR_I_OR_S_OR_SMALL_OR_STRIKE_OR_STRONG_OR_TT_OR_U),
    ("blockquote", "blockquote", DIV_OR_BLOCKQUOTE_OR_CENTER_OR_MENU | SPECIAL),
    ("body", "body", BODY | SPECIAL | OPTIONAL_END_TAG),
    ("br", "br", BR | SPECIAL),
    ("button", "button", BUTTON | SPECIAL),
    ("canvas", "canvas", OTHER),
    ("caption", "caption", CAPTION | SPECIAL | SCOPING),
    ("center", "center", DIV_OR_BLOCKQUOTE_OR_CENTER_OR_MENU | SPECIAL),
    ("circle", "circle", OTHER),
    ("cite", "cite", OTHER),
    ("clippath", "clipPath", OTHER),
    ("code", "code", B_OR_BIG_OR_CODE_OR_EM_OR_I_OR_S_OR_SMALL_OR_STRIKE_OR_STRONG_OR_TT_OR_U),
    ("col", "col", COL | SPECIAL),
    ("colgroup", "colgroup", COLGROUP | SPECIAL | OPTIONAL_END_TAG),
    ("data", "data", OTHER),
    ("datalist", "datalist", OTHER),
    ("dd", "dd", DD_OR_DT | SPECIAL | OPTIONAL_END_TAG),
    ("defs", "defs", OTHER),
    ("del", "del", OTHER),
    ("desc", "desc", FOREIGNOBJECT_OR_DESC | SCOPING_AS_SVG),
    ("details", "details", ADDRESS_OR_ARTICLE_OR_ASIDE_OR_DETAILS_OR_DIALOG_OR_DIR_OR_FIGCAPTION_OR_FIGURE_OR_FOOTER_OR_HEADER_OR_HGROUP_OR_MAIN_OR_NAV_OR_SEARCH_OR_SECTION_OR_SUMMARY | SPECIAL),
    ("dfn", "dfn", OTHER),
    ("dialog", "dialog", ADDRESS_OR_ARTICLE_OR_ASIDE_OR_DETAILS_OR_DIALOG_OR_DIR_OR_FIGCAPTION_OR_FIGURE_OR_FOOTER_OR_HEADER_OR_HGROUP_OR_MAIN_OR_NAV_OR_SEARCH_OR_SECTION_OR_SUMMARY | SPECIAL),
    ("dir", "dir", ADDRESS_OR_ARTICLE_OR_ASIDE_OR_DETAILS_OR_DIALOG_OR_DIR_OR_FIGCAPTION_OR_FIGURE_OR_FOOTER_OR_HEADER_OR_HGROUP_OR_MAIN_OR_NAV_OR_SEARCH_OR_SECTION_OR_SUMMARY | SPECIAL),
    ("div", "div", DIV_OR_BLOCKQUOTE_OR_CENTER_OR_MENU | SPECIAL),
    ("dl", "dl", UL_OR_OL_OR_DL | SPECIAL),
    ("dt", "dt", DD_OR_DT | SPECIAL | OPTIONAL_END_TAG),
    ("ellipse", "ellipse", OTHER),
    ("em", "em", B_OR_BIG_OR_CODE_OR_EM_OR_I_OR_S_OR_SMALL_OR_STRIKE_OR_STRONG_OR_TT_OR_U),
    ("embed", "embed", EMBED | SPECIAL),
    ("feblend", "feBlend", OTHER),
    ("fecolormatrix", "feColorMatrix", OTHER),
    ("fecomponenttransfer", "feComponentTransfer", OTHER),
    ("fecomposite", "feComposite", OTHER),
    ("feconvolvematrix", "feConvolveMatrix", OTHER),
    ("fediffuselighting", "feDiffuseLighting", OTHER),
    ("fedisplacementmap", "feDisplacementMap", OTHER),
    ("fedistantlight", "feDistantLight", OTHER),
    ("fedropshadow", "feDropShadow", OTHER),
    ("feflood", "feFlood", OTHER),
    ("fefunca", "feFuncA", OTHER),
    ("fefuncb", "feFuncB", OTHER),
    ("fefuncg", "feFuncG", OTHER),
    ("fefuncr", "feFuncR", OTHER),
    ("fegaussianblur", "feGaussianBlur", OTHER),
    ("feimage", "feImage", OTHER),
    ("femerge", "feMerge", OTHER),
    ("femergenode", "feMergeNode", OTHER),
    ("femorphology", "feMorphology", OTHER),
    ("feoffset", "feOffset", OTHER),
    ("fepointlight", "fePointLight", OTHER),
    ("fespecularlighting", "feSpecularLighting", OTHER),
    ("fespotlight", "feSpotLight", OTHER),
    ("fetile", "feTile", OTHER),
    ("feturbulence", "feTurbulence", OTHER),
    ("fieldset", "fieldset", FIELDSET | SPECIAL),
    ("figcaption", "figcaption", ADDRESS_OR_ARTICLE_OR_ASIDE_OR_DETAILS_OR_DIALOG_OR_DIR_OR_FIGCAPTION_OR_FIGURE_OR_FOOTER_OR_HEADER_OR_HGROUP_OR_MAIN_OR_NAV_OR_SEARCH_OR_SECTION_OR_SUMMARY | SPECIAL),
    ("figure", "figure", ADDRESS_OR_ARTICLE_OR_ASIDE_OR_DETAILS_OR_DIALOG_OR_DIR_OR_FIGCAPTION_OR_FIGURE_OR_FOOTER_OR_HEADER_OR_HGROUP_OR_MAIN_OR_NAV_OR_SEARCH_OR_SECTION_OR_SUMMARY | SPECIAL),
    ("filter", "filter", OTHER),
    ("font", "font", FONT),
    ("footer", "footer", ADDRESS_OR_ARTICLE_OR_ASIDE_OR_DETAILS_OR_DIALOG_OR_DIR_OR_FIGCAPTION_OR_FIGURE_OR_FOOTER_OR_HEADER_OR_HGROUP_OR_MAIN_OR_NAV_OR_SEARCH_OR_SECTION_OR_SUMMARY | SPECIAL),
    ("foreignobject", "foreignObject", FOREIGNOBJECT_OR_DESC | SCOPING_AS_SVG),
    ("form", "form", FORM | SPECIAL),
    ("frame", "frame", FRAME | SPECIAL),
    ("frameset", "frameset", FRAMESET | SPECIAL),
    ("g", "g", OTHER),
    ("glyphref", "glyphRef", OTHER),
    ("h1", "h1", H1_OR_H2_OR_H3_OR_H4_OR_H5_OR_H6 | SPECIAL),
    ("h2", "h2", H1_OR_H2_OR_H3_OR_H4_OR_H5_OR_H6 | SPECIAL),
    ("h3", "h3", H1_OR_H2_OR_H3_OR_H4_OR_H5_OR_H6 | SPECIAL),
    ("h4", "h4", H1_OR_H2_OR_H3_OR_H4_OR_H5_OR_H6 | SPECIAL),
    ("h5", "h5", H1_OR_H2_OR_H3_OR_H4_OR_H5_OR_H6 | SPECIAL),
    ("h6", "h6", H1_OR_H2_OR_H3_OR_H4_OR_H5_OR_H6 | SPECIAL),
    ("head", "head", HEAD | SPECIAL | OPTIONAL_END_TAG),
    ("header", "header", ADDRESS_OR_ARTICLE_OR_ASIDE_OR_DETAILS_OR_DIALOG_OR_DIR_OR_FIGCAPTION_OR_FIGURE_OR_FOOTER_OR_HEADER_OR_HGROUP_OR_MAIN_OR_NAV_OR_SEARCH_OR_SECTION_OR_SUMMARY | SPECIAL),
    ("hgroup", "hgroup", ADDRESS_OR_ARTICLE_OR_ASIDE_OR_DETAILS_OR_DIALOG_OR_DIR_OR_FIGCAPTION_OR_FIGURE_OR_FOOTER_OR_HEADER_OR_HGROUP_OR_MAIN_OR_NAV_OR_SEARCH_OR_SECTION_OR_SUMMARY | SPECIAL),
    ("hr", "hr", HR | SPECIAL),
    ("html", "html", HTML | SPECIAL | SCOPING | OPTIONAL_END_TAG),
    ("i", "i", B_OR_BIG_OR_CODE_OR_EM_OR_I_OR_S_OR_SMALL_OR_STRIKE_OR_STRONG_OR_TT_OR_U),
    ("iframe", "iframe", IFRAME | SPECIAL),
    ("image", "image", IMAGE),
    ("img", "img", IMG | SPECIAL),
    ("input", "input", INPUT | SPECIAL),
    ("ins", "ins", OTHER),
    ("kbd", "kbd", OTHER),
    ("keygen", "keygen", KEYGEN | SPECIAL),
    ("label", "label", OTHER),
    ("legend", "legend", OTHER),
    ("li", "li", LI | SPECIAL | OPTIONAL_END_TAG),
    ("line", "line", OTHER),
    ("lineargradient", "linearGradient", OTHER),
    ("link", "link", LINK_OR_BASEFONT_OR_BGSOUND | SPECIAL),
    ("listing", "listing", PRE_OR_LISTING | SPECIAL),
    ("main", "main", ADDRESS_OR_ARTICLE_OR_ASIDE_OR_DETAILS_OR_DIALOG_OR_DIR_OR_FIGCAPTION_OR_FIGURE_OR_FOOTER_OR_HEADER_OR_HGROUP_OR_MAIN_OR_NAV_OR_SEARCH_OR_SECTION_OR_SUMMARY | SPECIAL),
    ("malignmark", "malignmark", MGLYPH_OR_MALIGNMARK),
    ("map", "map", OTHER),
    ("mark", "mark", OTHER),
    ("marker", "marker", OTHER),
    ("marquee", "marquee", MARQUEE_OR_APPLET | SPECIAL | SCOPING),
    ("mask", "mask", OTHER),
    ("math", "math", MATH),
    ("menu", "menu", DIV_OR_BLOCKQUOTE_OR_CENTER_OR_MENU | SPECIAL),
    ("meta", "meta", META | SPECIAL),
    ("metadata", "metadata", OTHER),
    ("meter", "meter", OTHER),
    ("mglyph", "mglyph", MGLYPH_OR_MALIGNMARK),
    ("mi", "mi", MI_MO_MN_MS_MTEXT | SCOPING_AS_MATHML),
    ("mn", "mn", MI_MO_MN_MS_MTEXT | SCOPING_AS_MATHML),
    ("mo", "mo", MI_MO_MN_MS_MTEXT | SCOPING_AS_MATHML),
    ("mpath", "mpath", OTHER),
    ("ms", "ms", MI_MO_MN_MS_MTEXT | SCOPING_AS_MATHML),
    ("mtext", "mtext", MI_MO_MN_MS_MTEXT | SCOPING_AS_MATHML),
    ("nav", "nav", ADDRESS_OR_ARTICLE_OR_ASIDE_OR_DETAILS_OR_DIALOG_OR_DIR_OR_FIGCAPTION_OR_FIGURE_OR_FOOTER_OR_HEADER_OR_HGROUP_OR_MAIN_OR_NAV_OR_SEARCH_OR_SECTION_OR_SUMMARY | SPECIAL),
    ("nobr", "nobr", NOBR),
    ("noembed", "noembed", NOEMBED | SPECIAL),
    ("noframes", "noframes", NOFRAMES | SPECIAL),
    ("noscript", "noscript", NOSCRIPT | SPECIAL),
    ("object", "object", OBJECT | SPECIAL | SCOPING),
    ("ol", "ol", UL_OR_OL_OR_DL | SPECIAL),
    ("optgroup", "optgroup", OPTGROUP | OPTIONAL_END_TAG),
    ("option", "option", OPTION | OPTIONAL_END_TAG),
    ("output", "output", OUTPUT),
    ("p", "p", P | SPECIAL | OPTIONAL_END_TAG),
    ("param", "param", PARAM_OR_SOURCE_OR_TRACK | SPECIAL),
    ("path", "path", OTHER),
    ("pattern", "pattern", OTHER),
    ("picture", "picture", OTHER),
    ("plaintext", "plaintext", PLAINTEXT | SPECIAL),
    ("polygon", "polygon", OTHER),
    ("polyline", "polyline", OTHER),
    ("pre", "pre", PRE_OR_LISTING | SPECIAL),
    ("progress", "progress", OTHER),
    ("q", "q", OTHER),
    ("radialgradient", "radialGradient", OTHER),
    ("rb", "rb", RB_OR_RTC | OPTIONAL_END_TAG),
    ("rect", "rect", OTHER),
    ("rp", "rp", RT_OR_RP | OPTIONAL_END_TAG),
    ("rt", "rt", RT_OR_RP | OPTIONAL_END_TAG),
    ("rtc", "rtc", RB_OR_RTC | OPTIONAL_END_TAG),
    ("ruby", "ruby", RUBY_OR_SPAN_OR_SUB_OR_SUP_OR_VAR),
    ("s", "s", B_OR_BIG_OR_CODE_OR_EM_OR_I_OR_S_OR_SMALL_OR_STRIKE_OR_STRONG_OR_TT_OR_U),
    ("samp", "samp", OTHER),
    ("script", "script", SCRIPT | SPECIAL),
    ("search", "search", ADDRESS_OR_ARTICLE_OR_ASIDE_OR_DETAILS_OR_DIALOG_OR_DIR_OR_FIGCAPTION_OR_FIGURE_OR_FOOTER_OR_HEADER_OR_HGROUP_OR_MAIN_OR_NAV_OR_SEARCH_OR_SECTION_OR_SUMMARY | SPECIAL),
    ("section", "section", ADDRESS_OR_ARTICLE_OR_ASIDE_OR_DETAILS_OR_DIALOG_OR_DIR_OR_FIGCAPTION_OR_FIGURE_OR_FOOTER_OR_HEADER_OR_HGROUP_OR_MAIN_OR_NAV_OR_SEARCH_OR_SECTION_OR_SUMMARY | SPECIAL),
    ("select", "select", SELECT | SPECIAL | SCOPING),
    ("selectedcontent", "selectedcontent", OTHER),
    ("set", "set", OTHER),
    ("slot", "slot", OTHER),
    ("small", "small", B_OR_BIG_OR_CODE_OR_EM_OR_I_OR_S_OR_SMALL_OR_STRIKE_OR_STRONG_OR_TT_OR_U),
    ("source", "source", PARAM_OR_SOURCE_OR_TRACK),
    ("span", "span", RUBY_OR_SPAN_OR_SUB_OR_SUP_OR_VAR),
    ("stop", "stop", OTHER),
    ("strike", "strike", B_OR_BIG_OR_CODE_OR_EM_OR_I_OR_S_OR_SMALL_OR_STRIKE_OR_STRONG_OR_TT_OR_U),
    ("strong", "strong", B_OR_BIG_OR_CODE_OR_EM_OR_I_OR_S_OR_SMALL_OR_STRIKE_OR_STRONG_OR_TT_OR_U),
    ("style", "style", STYLE | SPECIAL),
    ("sub", "sub", RUBY_OR_SPAN_OR_SUB_OR_SUP_OR_VAR),
    ("summary", "summary", ADDRESS_OR_ARTICLE_OR_ASIDE_OR_DETAILS_OR_DIALOG_OR_DIR_OR_FIGCAPTION_OR_FIGURE_OR_FOOTER_OR_HEADER_OR_HGROUP_OR_MAIN_OR_NAV_OR_SEARCH_OR_SECTION_OR_SUMMARY | SPECIAL),
    ("sup", "sup", RUBY_OR_SPAN_OR_SUB_OR_SUP_OR_VAR),
    ("svg", "svg", SVG),
    ("switch", "switch", OTHER),
    ("symbol", "symbol", OTHER),
    ("table", "table", TABLE | SPECIAL | FOSTER_PARENTING | SCOPING),
    ("tbody", "tbody", TBODY_OR_THEAD_OR_TFOOT | SPECIAL | FOSTER_PARENTING | OPTIONAL_END_TAG),
    ("td", "td", TD_OR_TH | SPECIAL | SCOPING | OPTIONAL_END_TAG),
    ("template", "template", TEMPLATE | SPECIAL | SCOPING),
    ("text", "text", OTHER),
    ("textarea", "textarea", TEXTAREA | SPECIAL),
    ("textpath", "textPath", OTHER),
    ("tfoot", "tfoot", TBODY_OR_THEAD_OR_TFOOT | SPECIAL | FOSTER_PARENTING | OPTIONAL_END_TAG),
    ("th", "th", TD_OR_TH | SPECIAL | SCOPING | OPTIONAL_END_TAG),
    ("thead", "thead", TBODY_OR_THEAD_OR_TFOOT | SPECIAL | FOSTER_PARENTING | OPTIONAL_END_TAG),
    ("time", "time", OTHER),
    ("title", "title", TITLE | SPECIAL | SCOPING_AS_SVG),
    ("tr", "tr", TR | SPECIAL | FOSTER_PARENTING | OPTIONAL_END_TAG),
    ("track", "track", PARAM_OR_SOURCE_OR_TRACK | SPECIAL),
    ("tspan", "tspan", OTHER),
    ("tt", "tt", B_OR_BIG_OR_CODE_OR_EM_OR_I_OR_S_OR_SMALL_OR_STRIKE_OR_STRONG_OR_TT_OR_U),
    ("u", "u", B_OR_BIG_OR_CODE_OR_EM_OR_I_OR_S_OR_SMALL_OR_STRIKE_OR_STRONG_OR_TT_OR_U),
    ("ul", "ul", UL_OR_OL_OR_DL | SPECIAL),
    ("use", "use", OTHER),
    ("var", "var", RUBY_OR_SPAN_OR_SUB_OR_SUP_OR_VAR),
    ("video", "video", OTHER),
    ("view", "view", OTHER),
    ("wbr", "wbr", AREA_OR_WBR | SPECIAL),
    ("xmp", "xmp", XMP | SPECIAL),
];

/// Name, SVG camel-case name and flags of a known element, `None` for custom names.
pub(super) fn lookup(name: &str) -> Option<(&'static str, &'static str, u32)> {
    ELEMENT_NAMES
        .binary_search_by(|(n, _, _)| (*n).cmp(name))
        .ok()
        .map(|i| ELEMENT_NAMES[i])
}
