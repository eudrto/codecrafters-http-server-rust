use clap::Parser;

use request::Request;
use response_writer::ResponseWriter;
use router::Router;
use server::Server;
use status_code_registry::ReasonPhrase;

mod file_server;
mod request;
mod response_writer;
mod router;
mod server;
mod status_code_registry;
#[cfg(test)]
mod test_utils;

#[ctor::ctor]
fn init_tracing() {
    use tracing_subscriber::fmt::format;

    let builder = tracing_subscriber::fmt()
        .event_format(format().pretty())
        .with_target(false)
        .with_file(true)
        .with_line_number(true);

    #[cfg(not(test))]
    builder.init();
    #[cfg(test)]
    builder.with_test_writer().init();
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    directory: Option<String>,
}

fn home(w: &mut ResponseWriter, _: &mut Request) {
    w.set_reason_phrase(ReasonPhrase::OK);
}

fn echo(w: &mut ResponseWriter, r: &mut Request) {
    w.set_body_str(r.get_param().unwrap());
    w.set_reason_phrase(ReasonPhrase::OK);
}

fn user_agent(w: &mut ResponseWriter, r: &mut Request) {
    w.set_body_str(r.get_header("User-Agent").unwrap_or(""));
    w.set_reason_phrase(ReasonPhrase::OK);
}

pub fn run() {
    let args = Args::parse();

    let mut router = Router::new();
    router.add_route("/", &home);
    router.add_route("/echo/:str", &echo);
    router.add_route("/user-agent", &user_agent);

    let file_retriever = args
        .directory
        .as_deref()
        .map(|directory| file_server::new_file_retriever(directory));
    if let Some(file_retriever) = &file_retriever {
        router.add_route("/files/", file_retriever);
    };

    let server = Server::new("127.0.0.1:4221");
    server.run(router);
}

#[cfg(test)]
pub mod tests {
    use std::thread;

    use reqwest::{blocking::Client, header};

    use crate::{router::Router, server::Server};

    use super::{echo, home, user_agent};

    #[test]
    fn test_home() {
        let server = Server::new("localhost:0");
        let addr = server.local_addr();

        thread::spawn(move || {
            let mut router = Router::new();
            router.add_route("/", &home);
            server.run(router);
        });

        let url = format!("http://{}", addr);
        let resp = reqwest::blocking::get(url).unwrap();
        assert_eq!(resp.status(), 200);
    }

    #[test]
    fn test_echo() {
        let server = Server::new("localhost:0");
        let addr = server.local_addr();

        thread::spawn(move || {
            let mut router = Router::new();
            router.add_route("/echo/:str", &echo);
            server.run(router);
        });

        let url = format!("http://{}/echo/hello", addr);
        let resp = reqwest::blocking::get(url).unwrap();
        let body = resp.text().unwrap();
        assert_eq!(body, "hello");
    }

    #[test]
    fn test_user_agent() {
        let server = Server::new("localhost:0");
        let addr = server.local_addr();

        thread::spawn(move || {
            let mut router = Router::new();
            router.add_route("/user-agent", &user_agent);
            server.run(router);
        });

        let url = format!("http://{}/user-agent", addr);
        let resp = Client::new()
            .get(url)
            .header(header::USER_AGENT, "test")
            .send()
            .unwrap();

        let body = resp.text().unwrap();
        assert_eq!(body, "test");
    }
}
