use std::collections::HashMap;

use crate::server::Handler;

use super::Match;

pub struct Exact<'a>(HashMap<String, &'a (dyn Handler + Sync)>);

impl<'a> Exact<'a> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_route(&mut self, pattern: impl Into<String>, handler: &'a (impl Handler + Sync)) {
        let pattern = pattern.into();
        assert!(pattern == "/" || !pattern.ends_with("/"));
        self.0.insert(pattern, handler);
    }

    pub fn pattern_match(&self, request_target: &str) -> Option<Match> {
        self.0
            .get_key_value(request_target)
            .map(|(pattern, handler)| Match::new(pattern, *handler))
    }
}

#[cfg(test)]
mod tests {
    use crate::server::noop_handler;

    use super::Exact;

    #[test]
    fn test_exact() {
        let mut exact = Exact::new();

        let noop_handler = &noop_handler();
        exact.add_route("/".to_owned(), noop_handler);
        exact.add_route("/items", noop_handler);

        let m = exact.pattern_match("/").unwrap();
        assert_eq!(m.pattern, "/");

        let m = exact.pattern_match("/items").unwrap();
        assert_eq!(m.pattern, "/items");
    }

    #[test]
    fn test_exact_no_match() {
        let mut exact = Exact::new();

        let noop_handler = &noop_handler();
        exact.add_route("/items", noop_handler);

        assert!(exact.pattern_match("/items/").is_none());
    }
}
