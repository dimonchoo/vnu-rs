pub mod elements;
pub mod relaxng;

pub use elements::{
    STANDARD_HTML_ELEMENTS, VOID_ELEMENTS, is_standard_html_element, is_void_element,
};
