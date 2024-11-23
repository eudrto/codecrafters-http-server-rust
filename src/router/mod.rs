use matcher::{Chain, Match};
use tracing::info;

use crate::{
    request::Request,
    response_writer::ResponseWriter,
    server::{Handler, HttpMethod},
    status_code_registry::ReasonPhrase,
};

mod matcher;

pub struct Router<'a> {
    chains: [Chain<'a>; 2],
}

impl<'a> Router<'a> {
    pub fn new() -> Self {
        Self {
            chains: [Chain::new(), Chain::new()],
        }
    }

    pub fn add_route(
        &mut self,
        http_method: HttpMethod,
        pattern: impl Into<String>,
        handler: &'a (impl Handler + Sync),
    ) {
        self.chains[http_method as usize].add_route(pattern, handler);
    }

    fn pattern_match(
        &self,
        http_method: HttpMethod,
        request_target: &str,
    ) -> Option<(Match, Option<String>)> {
        self.chains[http_method as usize].pattern_match(request_target)
    }

    fn find_allowed_methods(&self, request_target: &str) -> Vec<HttpMethod> {
        self.chains
            .iter()
            .enumerate()
            .filter_map(|(idx, chain)| {
                if chain.pattern_match(request_target).is_some() {
                    Some(HttpMethod::try_from(idx).unwrap())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn handle(&self, w: &mut ResponseWriter, r: &mut Request) {
        let Ok(http_method) = HttpMethod::try_from(r.get_http_method()) else {
            w.set_reason_phrase(ReasonPhrase::BadRequest);
            return;
        };
        let request_target = r.get_request_target();

        if let Some((m, param)) = self.pattern_match(http_method, request_target) {
            info!("match: {}", m.pattern);
            if let Some(param) = param {
                r.set_param(param);
            }
            m.handler.handle(w, r);
            return;
        }

        let allowed_methods = self.find_allowed_methods(request_target);
        if !allowed_methods.is_empty() {
            w.add_allow_header(allowed_methods);
            w.set_reason_phrase(ReasonPhrase::MethodNotAllowed);
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
    use std::thread;

    use crate::server::{noop_handler, HttpMethod, Server};

    use super::Router;

    #[test]
    fn test_find_allowed_methods() {
        let mut router = Router::new();
        let noop_handler = &noop_handler();
        router.add_route(HttpMethod::Get, "/items", noop_handler);
        assert_eq!(router.find_allowed_methods("/items"), vec![HttpMethod::Get]);
    }

    #[test]
    fn test_method_not_allowed() {
        let server = Server::new("localhost:0");
        let addr = server.local_addr();

        thread::spawn(move || {
            let mut router = Router::new();
            let noop_handler = &noop_handler();
            router.add_route(HttpMethod::Post, "/todos", noop_handler);
            server.run(router);
        });

        let url = format!("http://{}/todos", addr);
        let resp = reqwest::blocking::get(url).unwrap();
        assert_eq!(resp.status(), 405);
        assert_eq!(resp.headers().get("allow").unwrap(), "POST");
    }
}
