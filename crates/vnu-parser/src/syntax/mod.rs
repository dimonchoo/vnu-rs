pub mod charrefs;
pub mod comments;
pub mod doctypes;
pub mod eof;
pub mod scanner;
pub mod tags;
pub mod tracker;

pub use scanner::scan_syntax;
pub use tracker::SyntaxTracker;
