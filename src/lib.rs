use response_writer::ResponseWriter;
use server::Server;
use status_code_registry::ReasonPhrase;

mod response_writer;
mod server;
mod status_code_registry;

pub fn run() {
    Server::run("127.0.0.1:4221", |w: &mut ResponseWriter| {
        w.set_reason_phrase(ReasonPhrase::OK);
    });
}
