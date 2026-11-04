pub mod aria;
pub mod body_closed;
pub mod content_model;
pub mod datatypes;
pub mod doctype;
pub mod duplicate_id;
pub mod forms;
pub mod head_title;
pub mod html_lang;
pub mod img_alt;
pub mod link;
pub mod lists;
pub mod media;
pub mod meta;
pub mod metadata;
pub mod nesting;
pub mod obsolete;
pub mod scripting;
pub mod tables;

pub use aria::AriaRule;
pub use body_closed::BodyClosedRule;
pub use content_model::ContentModelRule;
pub use datatypes::DatatypesRule;
pub use doctype::DoctypeRule;
pub use duplicate_id::DuplicateIdRule;
pub use forms::{FormsRule, InputRule};
pub use head_title::HeadTitleRule;
pub use html_lang::HtmlLangRule;
pub use img_alt::ImgAltRule;
pub use link::LinkRule;
pub use lists::DlRule;
pub use media::MediaRule;
pub use meta::MetaRule;
pub use metadata::MetadataRule;
pub use nesting::NestingRule;
pub use obsolete::ObsoleteRule;
pub use scripting::ScriptingRule;
pub use tables::TablesRule;

use crate::rule::Rule;

pub fn default_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(DoctypeRule::default()),
        Box::new(HtmlLangRule),
        Box::new(HeadTitleRule::default()),
        Box::new(ImgAltRule),
        Box::new(NestingRule),
        Box::new(DuplicateIdRule::default()),
        Box::new(FormsRule::default()),
        Box::new(InputRule::default()),
        Box::new(DatatypesRule),
        Box::new(ObsoleteRule),
        Box::new(ScriptingRule::default()),
        Box::new(MediaRule::default()),
        Box::new(DlRule::default()),
        Box::new(BodyClosedRule::default()),
        Box::new(TablesRule),
        Box::new(AriaRule::default()),
        Box::new(ContentModelRule),
        Box::new(MetadataRule::default()),
        Box::new(MetaRule::default()),
        Box::new(LinkRule),
    ]
}
