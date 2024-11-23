use std::collections::HashMap;

use crate::server::Handler;

pub struct Exact<'a>(HashMap<String, &'a (dyn Handler + Sync)>);

impl<'a> Exact<'a> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_route(&mut self, pattern: String, handler: &'a (impl Handler + Sync)) {
        assert!(pattern == "/" || !pattern.ends_with("/"));
        self.0.insert(pattern, handler);
    }

    pub fn pattern_match<'param>(&self, request_target: &str) -> Option<&(dyn Handler + Sync)> {
        self.0.get(request_target).map(|h| *h)
    }
}
