use crate::syntax::SyntaxTracker;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct ScanPolicy {
    pub tracker: SyntaxTracker,
}

impl ScanPolicy {
    #[inline]
    pub fn maybe_scan(&self, html: &str, messages: &mut Vec<VNuMessage>) {
        if self.tracker.needs_scan {
            crate::syntax::scan_syntax(html, messages);
        }
    }
}
