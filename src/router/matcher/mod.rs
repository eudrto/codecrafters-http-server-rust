pub use dynamic::Dynamic;
pub use exact::Exact;
pub use subtree::Subtree;

use crate::server::Handler;

mod dynamic;
mod exact;
mod subtree;

pub struct Match<'p, 'h> {
    pub pattern: &'p str,
    pub handler: &'h (dyn Handler + Sync),
}

impl<'p, 'h> Match<'p, 'h> {
    fn new(pattern: &'p str, handler: &'h (dyn Handler + Sync)) -> Self {
        Self {
            pattern: pattern,
            handler,
        }
    }
}
