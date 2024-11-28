use crate::server::Handler;

use super::Match;

pub struct Subtree<'a>(Vec<(String, &'a (dyn Handler + Sync))>);

impl<'a> Subtree<'a> {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn add_route(&mut self, pattern: impl Into<String>, handler: &'a (impl Handler + Sync)) {
        let pattern = pattern.into();
        assert!(pattern != "/" && pattern.ends_with("/"));
        self.0.push((pattern, handler));
        self.0.sort_by(|(l, _), (r, _)| r.cmp(l));
    }

    pub fn pattern_match<'req_line>(
        &self,
        request_target: &'req_line str,
    ) -> Option<(Match, &'req_line str)> {
        for (pattern, handler) in &self.0 {
            if let Some(param) = request_target.strip_prefix(pattern) {
                return Some((Match::new(pattern, *handler), param));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::server::noop_handler;

    use super::Subtree;

    #[test]
    fn test_subtree_match() {
        let mut subtree = Subtree::new();

        let pattern = "/fst/";
        let noop_handler = &noop_handler();
        subtree.add_route("/fst/", noop_handler);

        let tests = [
            ("/fst/", ""),
            ("/fst/xyz", "xyz"),
            ("/fst/xyz/", "xyz/"),
            ("/fst/xy/z", "xy/z"),
            ("/fst/xy/z/", "xy/z/"),
        ];

        for (request_target, param_want) in tests {
            let (m, param_got) = subtree.pattern_match(request_target).unwrap();
            assert_eq!(m.pattern, pattern);
            assert_eq!(param_got, param_want);
        }
    }

    #[test]
    fn test_subtree_no_match() {
        let mut subtree = Subtree::new();

        let noop_handler = &noop_handler();
        subtree.add_route("/fst/", noop_handler);

        assert!(subtree.pattern_match("/fst").is_none());
        assert!(subtree.pattern_match("/fstxyz").is_none());
    }
}
