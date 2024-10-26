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

fn home(w: &mut ResponseWriter, _: &Request) {
    w.set_reason_phrase(ReasonPhrase::OK);
}

pub fn run() {
    let mut router = Router::new();
    router.add_route("/".to_owned(), home);

    Server::run("127.0.0.1:4221", router);
}
