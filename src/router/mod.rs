use matcher::{Dynamic, Exact};

use crate::{
    request::Request, response_writer::ResponseWriter, server::Handler,
    status_code_registry::ReasonPhrase,
};

mod matcher;

pub struct Router<'a> {
    exact: Exact<'a>,
    dynamic: Dynamic<'a>,
}

impl<'a> Router<'a> {
    pub fn new() -> Self {
        Self {
            exact: Exact::new(),
            dynamic: Dynamic::new(),
        }
    }

    pub fn add_route(&mut self, pattern: impl Into<String>, handler: &'a (impl Handler + Sync)) {
        let mut pattern = pattern.into();
        assert!(pattern.starts_with('/'));

        if pattern == "/" {
            self.exact.add_route(pattern, handler);
            return;
        }
        if pattern.ends_with("/") {
            pattern.pop();
        }
        let (key, param) = pattern.rsplit_once("/").unwrap();
        if param.starts_with(":") {
            self.dynamic.add_route(key, handler);
            return;
        }
        self.exact.add_route(pattern, handler);
    }

    pub fn handle(&self, w: &mut ResponseWriter, r: &mut Request) {
        let mut uri = r.get_request_target();

        if uri.ends_with("/") && uri != "/" {
            uri = &uri[..uri.len() - 1];
        }

        if let Some((param, handler)) = self.dynamic.pattern_match(uri) {
            r.set_param(param);
            handler.handle(w, r);
            return;
        }
        if let Some(handler) = self.exact.pattern_match(uri) {
            handler.handle(w, r);
            return;
        }

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
    use std::collections::HashMap;

    use crate::{request::Request, response_writer::ResponseWriter, server::noop_handler};

    use super::Router;

    fn run(router: &Router, uri: &str) -> (ResponseWriter, Request) {
        let mut w = ResponseWriter::new_empty();
        let status_line = format!("GET {} HTTP/1.1\r\n\r\n", uri);
        let mut r = Request::new(status_line, None, HashMap::new());
        router.handle(&mut w, &mut r);
        (w, r)
    }

    #[test]
    fn test_not_found() {
        let mut router = Router::new();
        let noop_handler = &noop_handler();
        router.add_route("/", noop_handler);
        router.add_route("/items", noop_handler);

        struct Test {
            uri: &'static str,
            status_code: Option<u16>,
        }

        let tests = [
            Test {
                uri: "/",
                status_code: None,
            },
            Test {
                uri: "/items",
                status_code: None,
            },
            Test {
                uri: "/about",
                status_code: Some(404),
            },
        ];

        for test in tests {
            let (w, _) = run(&router, test.uri);
            assert_eq!(w.get_status_code(), test.status_code);
        }
    }

    #[test]
    fn test_router_dynamic() {
        let mut router = Router::new();
        let noop_handler = &noop_handler();
        router.add_route("/", noop_handler);
        router.add_route("/items/:id", noop_handler);

        struct Test {
            uri: &'static str,
            status_code: Option<u16>,
            param: Option<&'static str>,
        }

        let tests = [
            Test {
                uri: "/",
                status_code: None,
                param: None,
            },
            Test {
                uri: "/items",
                status_code: Some(404),
                param: None,
            },
            Test {
                uri: "/items/",
                status_code: Some(404),
                param: None,
            },
            Test {
                uri: "/items/xyz",
                status_code: None,
                param: Some("xyz"),
            },
            Test {
                uri: "/items/xyz/",
                status_code: None,
                param: Some("xyz"),
            },
            Test {
                uri: "/items/xyz/a",
                status_code: Some(404),
                param: None,
            },
        ];

        for test in tests {
            let (w, r) = run(&router, test.uri);
            assert_eq!(w.get_status_code(), test.status_code);
            assert_eq!(r.get_param(), test.param);
        }
    }

    #[test]
    fn test_router_both() {
        let mut router = Router::new();
        let noop_handler = &noop_handler();
        router.add_route("/", noop_handler);
        router.add_route("/items", noop_handler);
        router.add_route("/items/:id", noop_handler);

        struct Test {
            uri: &'static str,
            status_code: Option<u16>,
            param: Option<&'static str>,
        }

        let tests = [
            Test {
                uri: "/",
                status_code: None,
                param: None,
            },
            Test {
                uri: "/items",
                status_code: None,
                param: None,
            },
            Test {
                uri: "/items/",
                status_code: None,
                param: None,
            },
            Test {
                uri: "/items/xyz",
                status_code: None,
                param: Some("xyz"),
            },
            Test {
                uri: "/items/xyz/",
                status_code: None,
                param: Some("xyz"),
            },
            Test {
                uri: "/items/xyz/a",
                status_code: Some(404),
                param: None,
            },
        ];

        for test in tests {
            let (w, r) = run(&router, test.uri);
            assert_eq!(w.get_status_code(), test.status_code);
            assert_eq!(r.get_param(), test.param);
        }
    }
}
