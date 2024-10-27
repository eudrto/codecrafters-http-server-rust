use std::collections::HashMap;

use crate::{
    request::Request, response_writer::ResponseWriter, server::Handler,
    status_code_registry::ReasonPhrase,
};

pub struct Router {
    exact: HashMap<String, Box<dyn Handler>>,
    dynamic: HashMap<String, Box<dyn Handler>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            exact: HashMap::new(),
            dynamic: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, mut pattern: String, handler: impl Handler + 'static) {
        assert!(pattern.starts_with('/'));

        let handler = Box::new(handler);

        if pattern == "/" {
            self.exact.insert(pattern, handler);
            return;
        }

        if pattern.ends_with("/") {
            pattern.pop();
        }

        let (prefix, suffix) = pattern.rsplit_once("/").unwrap();
        if suffix.starts_with(":") {
            self.dynamic.insert(prefix.to_owned(), handler);
            return;
        }
        self.exact.insert(pattern, handler);
    }

    pub fn handle(&self, w: &mut ResponseWriter, r: &mut Request) {
        let mut uri = r.get_request_target();

        if uri.ends_with("/") && uri != "/" {
            uri = &uri[..uri.len() - 1];
        }

        let (prefix, param) = uri.rsplit_once("/").unwrap();
        if let Some(handler) = self.dynamic.get(prefix) {
            r.set_param(param.to_string());
            handler.handle(w, r);
            return;
        }
        if let Some(handler) = self.exact.get(uri) {
            handler.handle(w, r);
            return;
        }

        w.set_reason_phrase(ReasonPhrase::NotFound);
    }
}

impl Handler for Router {
    fn handle(&self, w: &mut ResponseWriter, r: &mut Request) {
        self.handle(w, r);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{request::Request, response_writer::ResponseWriter, server::Handler};

    use super::Router;

    fn noop_handler() -> impl Handler {
        |_: &mut ResponseWriter, _: &mut Request| {}
    }

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
        router.add_route("/".to_owned(), noop_handler());
        router.add_route("/items".to_owned(), noop_handler());

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
    fn test_router_root() {
        let mut router = Router::new();
        router.add_route("/".to_owned(), noop_handler());
        assert_eq!(router.exact.len(), 1);
        assert!(router.exact.contains_key("/"));
    }

    #[test]
    fn test_router_dynamic() {
        let mut router = Router::new();
        router.add_route("/".to_owned(), noop_handler());
        router.add_route("/items/:id".to_owned(), noop_handler());

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
        router.add_route("/".to_owned(), noop_handler());
        router.add_route("/items".to_owned(), noop_handler());
        router.add_route("/items/:id".to_owned(), noop_handler());

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
