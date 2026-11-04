pub mod doctype;
pub mod duplicate_id;
pub mod forms;
pub mod head_title;
pub mod html_lang;
pub mod img_alt;
pub mod nesting;
pub mod obsolete;
pub mod tables;

pub use doctype::DoctypeRule;
pub use duplicate_id::DuplicateIdRule;
pub use forms::FormsRule;
pub use head_title::HeadTitleRule;
pub use html_lang::HtmlLangRule;
pub use img_alt::ImgAltRule;
pub use nesting::NestingRule;
pub use obsolete::ObsoleteRule;
pub use tables::TablesRule;

use crate::rule::Rule;

pub fn default_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(DoctypeRule::default()),
        Box::new(HtmlLangRule::default()),
        Box::new(HeadTitleRule::default()),
        Box::new(ImgAltRule::default()),
        Box::new(NestingRule::default()),
        Box::new(DuplicateIdRule::default()),
        Box::new(FormsRule::default()),
        Box::new(ObsoleteRule::default()),
        Box::new(TablesRule::default()),
    ]
}
