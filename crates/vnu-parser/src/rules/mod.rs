pub mod aria;
pub mod content_model;
pub mod doctype;
pub mod duplicate_id;
pub mod forms;
pub mod head_title;
pub mod html_lang;
pub mod img_alt;
pub mod metadata;
pub mod nesting;
pub mod obsolete;
pub mod tables;

pub use aria::AriaRule;
pub use content_model::ContentModelRule;
pub use doctype::DoctypeRule;
pub use duplicate_id::DuplicateIdRule;
pub use forms::FormsRule;
pub use head_title::HeadTitleRule;
pub use html_lang::HtmlLangRule;
pub use img_alt::ImgAltRule;
pub use metadata::MetadataRule;
pub use nesting::NestingRule;
pub use obsolete::ObsoleteRule;
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
        Box::new(FormsRule),
        Box::new(ObsoleteRule),
        Box::new(TablesRule),
        Box::new(AriaRule::default()),
        Box::new(ContentModelRule),
        Box::new(MetadataRule::default()),
    ]
}
