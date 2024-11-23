use std::collections::HashMap;

use crate::server::Handler;

pub struct Dynamic<'a>(HashMap<String, &'a (dyn Handler + Sync)>);

impl<'a> Dynamic<'a> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_route(&mut self, prefix: String, handler: &'a (impl Handler + Sync)) {
        assert!(!prefix.ends_with("/"));
        self.0.insert(prefix, handler);
    }

    pub fn pattern_match(&self, request_target: &str) -> Option<(String, &(dyn Handler + Sync))> {
        let (prefix, param) = request_target.rsplit_once("/")?;
        if param.is_empty() {
            return None;
        }

        let handler = self.0.get(prefix)?;
        Some((param.to_owned(), *handler))
    }
}
