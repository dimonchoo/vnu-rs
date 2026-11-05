pub mod aria;
pub mod content_model;
pub mod custom_elements;
pub mod datatypes;
pub mod document;
pub mod figure;
pub mod foreign;
pub mod forms;
pub mod headings;
pub mod hyperlinks;
pub mod img_alt;
pub mod langdetect;
pub mod link;
pub mod lists;
pub mod media;
pub mod meta;
pub mod nesting;
pub mod obsolete;
pub mod scripting;
pub mod style;
pub mod tables;

pub use aria::AriaRule;
pub use content_model::ContentModelRule;
pub use custom_elements::CustomElementsRule;
pub use datatypes::DatatypesRule;
pub use document::{
    BodyClosedRule, CspRule, DoctypeRule, DuplicateIdRule, HeadTitleRule, HtmlLangRule,
    MetadataRule, RdfaRule, XmlStylesheetRule,
};
pub use figure::FigureRule;
pub use foreign::{ForeignMathRule, ForeignSvgRule};
pub use forms::{FormsRule, InputRule, LabelRule, SelectRule};
pub use headings::HeadingsRule;
pub use hyperlinks::HyperlinksRule;
pub use img_alt::ImgAltRule;
pub use langdetect::LangDetectRule;
pub use link::LinkRule;
pub use lists::{DlRule, LiRule, MenuRule};
pub use media::MediaRule;
pub use meta::MetaRule;
pub use nesting::NestingRule;
pub use obsolete::ObsoleteRule;
pub use scripting::ScriptingRule;
pub use style::StyleRule;
pub use tables::TablesRule;

use crate::rule::Rule;

pub fn default_rules() -> Vec<Box<dyn Rule>> {
    rules_with_options(true)
}

pub fn rules_with_options(enable_langdetect: bool) -> Vec<Box<dyn Rule>> {
    let mut rules: Vec<Box<dyn Rule>> = vec![Box::new(DoctypeRule::default())];
    rules.push(Box::new(HtmlLangRule::default()));
    rules.extend(vec![
        Box::new(HeadTitleRule::default()) as Box<dyn Rule>,
        Box::new(ImgAltRule),
        Box::new(HyperlinksRule),
        Box::new(NestingRule::default()),
        Box::new(DuplicateIdRule::default()),
        Box::new(FormsRule::default()),
        Box::new(InputRule::default()),
        Box::new(SelectRule::default()),
        Box::new(LabelRule::default()),
        Box::new(DatatypesRule::default()),
        Box::new(ObsoleteRule),
        Box::new(ScriptingRule::default()),
        Box::new(MediaRule::default()),
        Box::new(DlRule::default()),
        Box::new(LiRule),
        Box::new(MenuRule),
        Box::new(FigureRule::default()),
        Box::new(BodyClosedRule::default()),
        Box::new(TablesRule::default()),
        Box::new(AriaRule::default()),
        Box::new(ContentModelRule::default()),
        Box::new(CustomElementsRule),
        Box::new(MetadataRule::default()),
        Box::new(MetaRule::default()),
        Box::new(LinkRule),
        Box::new(StyleRule::default()),
        Box::new(HeadingsRule::default()),
        Box::new(ForeignMathRule::default()),
        Box::new(ForeignSvgRule::default()),
        Box::new(RdfaRule),
        Box::new(CspRule::default()),
    ]);
    if enable_langdetect {
        rules.push(Box::new(LangDetectRule::default()));
    }
    rules.push(Box::new(XmlStylesheetRule));
    rules
}

#[cfg(test)]
mod tests {
    use super::rules_with_options;
    use crate::engine::ValidatorEngine;

    #[test]
    fn no_langdetect_keeps_html_lang_and_drops_detector() {
        let without: Vec<&str> = rules_with_options(false)
            .iter()
            .map(|rule| rule.name())
            .collect();
        assert!(without.contains(&"html-lang"));
        assert!(!without.contains(&"langdetect"));

        let with: Vec<&str> = rules_with_options(true)
            .iter()
            .map(|rule| rule.name())
            .collect();
        assert!(with.contains(&"html-lang"));
        assert!(with.contains(&"langdetect"));
    }

    #[test]
    fn no_langdetect_still_warns_when_html_lang_is_missing() {
        let mut engine = ValidatorEngine::with_options(false);
        let report = engine
            .validate("<!DOCTYPE html><html><head><title>T</title></head><body></body></html>");
        assert!(
            report
                .messages
                .iter()
                .any(|message| message.message.contains("lang")),
            "missing html lang warning should survive --no-langdetect: {:?}",
            report.messages
        );
    }
}
