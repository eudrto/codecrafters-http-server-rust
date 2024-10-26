use std::collections::HashMap;

use crate::{
    request::Request, response_writer::ResponseWriter, server::Handler,
    status_code_registry::ReasonPhrase,
};

pub struct Router {
    routes: HashMap<String, Box<dyn Handler>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, path: String, handler: impl Handler + 'static) {
        self.routes.insert(path, Box::new(handler));
    }

    pub fn handle(&self, w: &mut ResponseWriter, r: &Request) {
        let uri = r.get_request_target();
        let Some(handler) = self.routes.get(uri) else {
            w.set_reason_phrase(ReasonPhrase::NotFound);
            return;
        };

        handler.handle(w, r);
    }
}

impl Handler for Router {
    fn handle(&self, w: &mut ResponseWriter, r: &Request) {
        self.handle(w, r);
    }
}

#[cfg(test)]
mod tests {
    use crate::{request::Request, response_writer::ResponseWriter, server::Handler};

    use super::Router;

    fn noop_handler() -> impl Handler {
        |_: &mut ResponseWriter, _: &Request| {}
    }

    fn run(router: &Router, uri: &str) -> ResponseWriter {
        let mut w = ResponseWriter::new_empty();
        let status_line = format!("GET {} HTTP/1.1\r\n\r\n", uri);
        let mut r = Request::new(status_line);
        router.handle(&mut w, &mut r);
        w
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
            let w = run(&router, test.uri);
            assert_eq!(w.get_status_code(), test.status_code);
        }
    }
}
