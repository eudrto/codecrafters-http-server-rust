use std::collections::HashMap;

use crate::server::Handler;

use super::Match;

pub struct Dynamic<'a>(HashMap<String, (String, &'a (dyn Handler + Sync))>);

impl<'a> Dynamic<'a> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_route(
        &mut self,
        prefix: impl Into<String>,
        pattern: impl Into<String>,
        handler: &'a (impl Handler + Sync),
    ) {
        let prefix = prefix.into();
        let pattern = pattern.into();
        assert!(!prefix.ends_with("/"));
        self.0.insert(prefix, (pattern, handler));
    }

    pub fn pattern_match<'req_line>(
        &self,
        request_target: &'req_line str,
    ) -> Option<(Match, &'req_line str)> {
        let (prefix, param) = request_target.rsplit_once("/")?;
        if param.is_empty() {
            return None;
        }

        self.0
            .get(prefix)
            .map(|(pattern, handler)| (Match::new(pattern, *handler), param))
    }
}

#[cfg(test)]
mod tests {
    use crate::server::noop_handler;

    use super::Dynamic;

    #[test]
    fn test_dynamic_match() {
        let mut dynamic = Dynamic::new();

        let noop_handler = &noop_handler();
        dynamic.add_route("/items", "/items/:id", noop_handler);

        let (m, param) = dynamic.pattern_match("/items/xyz").unwrap();
        assert_eq!(m.pattern, "/items/:id");
        assert_eq!(param, "xyz");
    }

    #[test]
    fn test_dynamic_no_match() {
        let mut dynamic = Dynamic::new();

        let noop_handler = &noop_handler();
        dynamic.add_route("/items", "/items/:id", noop_handler);

        assert!(dynamic.pattern_match("/items").is_none());
        assert!(dynamic.pattern_match("/items/").is_none());
        assert!(dynamic.pattern_match("/items/xyz/").is_none());
        assert!(dynamic.pattern_match("/items/xy/z").is_none());
        assert!(dynamic.pattern_match("/itemsxyz").is_none());
    }
}
