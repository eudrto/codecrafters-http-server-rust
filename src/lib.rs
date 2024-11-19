use request::Request;
use response_writer::ResponseWriter;
use router::Router;
use server::Server;
use status_code_registry::ReasonPhrase;

mod request;
mod response_writer;
mod router;
mod server;
mod status_code_registry;
#[cfg(test)]
mod test_utils;

fn home(w: &mut ResponseWriter, _: &mut Request) {
    w.set_reason_phrase(ReasonPhrase::OK);
}

fn echo(w: &mut ResponseWriter, r: &mut Request) {
    w.set_body_str(r.get_param().unwrap());
    w.set_reason_phrase(ReasonPhrase::OK);
}

fn user_agent(w: &mut ResponseWriter, r: &mut Request) {
    w.set_body_str(r.get_header("User-Agent"));
    w.set_reason_phrase(ReasonPhrase::OK);
}

pub fn run() {
    let mut router = Router::new();
    router.add_route("/".to_owned(), home);
    router.add_route("/echo/:str".to_owned(), echo);
    router.add_route("/user-agent".to_owned(), user_agent);

    Server::run("127.0.0.1:4221", router);
}
