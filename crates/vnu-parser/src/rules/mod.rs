pub mod doctype;
pub mod duplicate_id;
pub mod head_title;
pub mod html_lang;
pub mod img_alt;
pub mod nesting;

pub use doctype::DoctypeRule;
pub use duplicate_id::DuplicateIdRule;
pub use head_title::HeadTitleRule;
pub use html_lang::HtmlLangRule;
pub use img_alt::ImgAltRule;
pub use nesting::NestingRule;

use crate::rule::Rule;

pub fn default_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(DoctypeRule::default()),
        Box::new(HtmlLangRule::default()),
        Box::new(HeadTitleRule::default()),
        Box::new(ImgAltRule::default()),
        Box::new(NestingRule::default()),
        Box::new(DuplicateIdRule::default()),
    ]
}
