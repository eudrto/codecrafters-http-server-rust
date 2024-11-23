use matcher::{Dynamic, Exact, Match, Subtree};
use tracing::info;

use crate::{
    request::Request, response_writer::ResponseWriter, server::Handler,
    status_code_registry::ReasonPhrase,
};

mod matcher;

pub struct Router<'a> {
    exact: Exact<'a>,
    dynamic: Dynamic<'a>,
    subtree: Subtree<'a>,
}

impl<'a> Router<'a> {
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

    fn pattern_match<'param>(
        &self,
        request_target: &'param str,
    ) -> Option<(Match, Option<String>)> {
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

    pub fn handle(&self, w: &mut ResponseWriter, r: &mut Request) {
        let request_target = r.get_request_target();

        if let Some((m, param)) = self.pattern_match(request_target) {
            info!("match: {}", m.pattern);
            if let Some(param) = param {
                r.set_param(param);
            }
            m.handler.handle(w, r);
            return;
        }

        info!("no match");
        w.set_reason_phrase(ReasonPhrase::NotFound);
    }
}

impl<'a> Handler for Router<'a> {
    fn handle(&self, w: &mut ResponseWriter, r: &mut Request) {
        self.handle(w, r);
    }
}

#[cfg(test)]
mod tests {
    use crate::server::noop_handler;

    use super::Router;

    #[test]
    fn test_chain_match() {
        let mut chain = Router::new();

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
        let mut chain = Router::new();

        let noop_handler = &noop_handler();
        chain.add_route("/", noop_handler);

        assert!(chain.pattern_match("/hello").is_none());
    }
}
