use super::names::groups::*;
use super::*;

impl TreeBuilder {
    pub(super) fn append_html_element_to_document_and_push(&mut self, attrs: Attrs) {
        let node = self.html_node(&ElemName::lookup("html"), attrs);
        self.push(node);
    }

    pub(super) fn append_to_current_node_and_push_head_element(&mut self, attrs: Attrs) {
        self.head_pointer = true;
        let node = self.html_node(&ElemName::lookup("head"), attrs);
        self.push(node);
    }

    pub(super) fn append_to_current_node_and_push_body_element(&mut self, attrs: Attrs) {
        let node = self.html_node(&ElemName::lookup("body"), attrs);
        self.push(node);
    }

    pub(super) fn append_to_current_node_and_push_element(&mut self, en: &ElemName, attrs: Attrs) {
        let node = self.html_node(en, attrs);
        self.push(node);
    }

    pub(super) fn append_to_current_node_and_push_element_may_foster(
        &mut self,
        en: &ElemName,
        attrs: Attrs,
    ) {
        if self.current_fosters() {
            self.fatal();
        }
        let node = self.html_node(en, attrs);
        self.push(node);
    }

    pub(super) fn append_to_current_node_and_push_formatting_element_may_foster(
        &mut self,
        en: &ElemName,
        attrs: Attrs,
    ) {
        if self.current_fosters() {
            self.fatal();
        }
        let node = self.html_node(en, attrs);
        self.push(node.clone());
        self.list.push(Some(node));
    }

    pub(super) fn append_to_current_node_and_push_form_element_may_foster(&mut self, attrs: Attrs) {
        if self.current_fosters() {
            self.fatal();
        }
        if !self.is_template_contents() {
            self.form_pointer = true;
        }
        let node = self.html_node(&ElemName::lookup("form"), attrs);
        self.push(node);
    }

    pub(super) fn append_to_current_node_and_push_template_element(&mut self, attrs: Attrs) {
        let node = self.html_node(&ElemName::lookup("template"), attrs);
        self.push(node);
    }

    pub(super) fn append_to_current_node_and_push_foreign(
        &mut self,
        en: &ElemName,
        attrs: Attrs,
        ns: Ns,
    ) {
        if self.current_fosters() {
            self.fatal();
        }
        let (flags, pop_name) = match ns {
            Ns::Svg => (svg_flags(en.flags), en.camel.clone()),
            _ => {
                let integration = en.name == "annotation-xml"
                    && (equals_ci("application/xhtml+xml", attr(&attrs, "encoding"))
                        || equals_ci("text/html", attr(&attrs, "encoding")));
                (math_flags(en.flags, integration), en.name.clone())
            }
        };
        let node = Rc::new(StackNode {
            flags,
            name: en.name.clone(),
            pop_name,
            ns,
            attrs,
            loc: self.new_loc(),
        });
        self.push(node);
    }

    pub(super) fn append_void_element_to_current_may_foster(
        &mut self,
        en: &ElemName,
        attrs: Attrs,
    ) {
        if self.current_fosters() {
            self.fatal();
        }
        self.push_pop_void(Ns::Html, en.name.clone(), attrs);
    }

    pub(super) fn append_void_foreign(&mut self, en: &ElemName, attrs: Attrs, ns: Ns) {
        if self.current_fosters() {
            self.fatal();
        }
        let name = if ns == Ns::Svg {
            en.camel.clone()
        } else {
            en.name.clone()
        };
        self.push_pop_void(ns, name, attrs);
    }

    pub(super) fn push_head_pointer_onto_stack(&mut self) {
        self.fatal();
        let node = self.html_node(&ElemName::lookup("head"), no_attrs());
        self.silent_push(node);
    }

    pub(super) fn add_attributes_to(&mut self, pos: usize, attrs: &[(String, String)]) {
        let existing = self.stack[pos].attrs.clone();
        if attrs
            .iter()
            .any(|(k, _)| !existing.iter().any(|(ek, _)| ek == k))
        {
            self.fatal();
        }
    }

    pub(super) fn add_attributes_to_body(&mut self, attrs: &[(String, String)]) -> bool {
        if self.stack.len() >= 2 && self.stack[1].group() == BODY {
            self.add_attributes_to(1, attrs);
            return true;
        }
        false
    }

    pub(super) fn add_attributes_to_html(&mut self, attrs: &[(String, String)]) {
        self.add_attributes_to(0, attrs);
    }
}

fn svg_flags(flags: u32) -> u32 {
    let mut flags = flags & !(FOSTER_PARENTING | SCOPING | SPECIAL | OPTIONAL_END_TAG);
    if flags & SCOPING_AS_SVG != 0 {
        flags |= SCOPING | SPECIAL | HTML_INTEGRATION_POINT;
    }
    flags
}

fn math_flags(flags: u32, integration_point: bool) -> u32 {
    let mut flags = flags & !(FOSTER_PARENTING | SCOPING | SPECIAL | OPTIONAL_END_TAG);
    if flags & SCOPING_AS_MATHML != 0 {
        flags |= SCOPING | SPECIAL;
    }
    if integration_point {
        flags |= HTML_INTEGRATION_POINT;
    }
    flags
}
