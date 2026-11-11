pub mod csp;
pub mod duplicate_id;
pub mod head_title;
pub mod html_lang;
pub mod metadata;
pub mod rdfa;
pub mod xml_pi;

pub use csp::CspRule;
pub use duplicate_id::DuplicateIdRule;
pub use head_title::HeadTitleRule;
pub use html_lang::HtmlLangRule;
pub use metadata::MetadataRule;
pub use rdfa::RdfaRule;
pub use xml_pi::XmlStylesheetRule;
