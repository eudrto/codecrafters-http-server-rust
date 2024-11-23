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

        w.set_reason_phrase(ReasonPhrase::NotFound);
    }
}

impl<'a> Handler for Router<'a> {
    fn handle(&self, w: &mut ResponseWriter, r: &mut Request) {
        self.handle(w, r);
    }
}
