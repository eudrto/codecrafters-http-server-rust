use std::{
    io::Write,
    net::{TcpListener, ToSocketAddrs},
};

use crate::{
    request::{Request, RequestReader},
    response_writer::ResponseWriter,
    status_code_registry::ReasonPhrase,
};

#[derive(Debug)]
pub struct Server;

impl Server {
    pub fn run(addr: impl ToSocketAddrs, handler: impl Handler) {
        let listener = TcpListener::bind(addr).unwrap();

        for stream in listener.incoming() {
            let mut stream = match stream {
                Ok(stream) => stream,
                Err(err) => {
                    eprintln!("{:?}", err);
                    continue;
                }
            };

            let mut r = match RequestReader::new(&mut stream).read() {
                Ok(r) => r,
                Err(err) => {
                    eprintln!("{:?}", err);
                    let mut w = ResponseWriter::new_empty();
                    w.set_reason_phrase(ReasonPhrase::BadRequest);
                    if let Err(err) = stream.write_all(&w.write()) {
                        eprintln!("{:?}", err);
                    }
                    continue;
                }
            };
            dbg!(&r);

            let mut w = ResponseWriter::new_empty();
            handler.handle(&mut w, &mut r);
            let response = w.write();
            if let Err(err) = stream.write_all(&response) {
                eprintln!("{:?}", err);
            }
        }
    }
}

pub trait Handler {
    fn handle(&self, w: &mut ResponseWriter, r: &mut Request);
}

impl<T> Handler for T
where
    T: Fn(&mut ResponseWriter, &mut Request),
{
    fn handle(&self, w: &mut ResponseWriter, r: &mut Request) {
        self(w, r)
    }
}
