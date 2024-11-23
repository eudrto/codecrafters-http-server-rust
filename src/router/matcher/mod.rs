pub use dynamic::Dynamic;
pub use exact::Exact;

use crate::server::Handler;

mod dynamic;
mod exact;

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
