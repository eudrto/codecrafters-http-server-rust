use crate::server::Handler;

use super::{Dynamic, Exact, Match, Subtree};

pub struct Chain<'a> {
    exact: Exact<'a>,
    dynamic: Dynamic<'a>,
    subtree: Subtree<'a>,
}

impl<'a> Chain<'a> {
    pub fn new() -> Self {
        Self {
            exact: Exact::new(),
            dynamic: Dynamic::new(),
            subtree: Subtree::new(),
        }
    }

    pub fn add_route(&mut self, pattern: impl Into<String>, handler: &'a (impl Handler + Sync)) {
        let pattern = pattern.into();
        assert!(pattern.starts_with('/'));

        if pattern == "/" {
            self.exact.add_route(pattern, handler);
            return;
        }
        if pattern.ends_with("/") {
            self.subtree.add_route(pattern, handler);
            return;
        }
        let (key, param) = pattern.rsplit_once("/").unwrap();
        if param.starts_with(":") {
            self.dynamic.add_route(key, &pattern, handler);
            return;
        }
        self.exact.add_route(pattern, handler);
    }

    pub fn pattern_match<'req_line>(
        &self,
        request_target: &'req_line str,
    ) -> Option<(Match, Option<&'req_line str>)> {
        if let Some(m) = self.exact.pattern_match(request_target) {
            return Some((m, None));
        }
        if let Some((m, param)) = self.dynamic.pattern_match(request_target) {
            return Some((m, Some(param)));
        }
        if let Some((m, param)) = self.subtree.pattern_match(request_target) {
            return Some((m, Some(param)));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::server::noop_handler;

    use super::Chain;

    #[test]
    fn test_chain_match() {
        let mut chain = Chain::new();

        let noop_handler = &noop_handler();
        chain.add_route("/", noop_handler);
        chain.add_route("/fst", noop_handler);
        chain.add_route("/fst/:var", noop_handler);
        chain.add_route("/fst/", noop_handler);
        chain.add_route("/fst/snd/", noop_handler);

        let tests = [
            // "/"
            ("/", "/", None),
            // "/fst"
            ("/fst", "/fst", None),
            // "/fst/:var"
            ("/fst/val", "/fst/:var", Some("val")),
            ("/fst/snd", "/fst/:var", Some("snd")),
            // "/fst/"
            ("/fst/", "/fst/", Some("")),
            ("/fst/ab/c", "/fst/", Some("ab/c")),
            // "/fst/snd/"
            ("/fst/snd/", "/fst/snd/", Some("")),
            ("/fst/snd/abc", "/fst/snd/", Some("abc")),
            ("/fst/snd/ab/c", "/fst/snd/", Some("ab/c")),
        ];

        for (request_target, pattern, param_want) in tests {
            let (m, param_got) = chain.pattern_match(request_target).unwrap();
            assert_eq!(m.pattern, pattern);
            assert_eq!(param_got.as_deref(), param_want);
        }
    }

    #[test]
    fn test_chain_no_match() {
        let mut chain = Chain::new();

        let noop_handler = &noop_handler();
        chain.add_route("/", noop_handler);

        assert!(chain.pattern_match("/hello").is_none());
    }
}
