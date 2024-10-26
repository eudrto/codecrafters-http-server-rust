use std::{
    io::Write,
    net::{TcpListener, ToSocketAddrs},
};

use crate::{
    request::{Request, RequestReader},
    response_writer::ResponseWriter,
};

#[derive(Debug)]
pub struct Server;

impl Server {
    pub fn run(addr: impl ToSocketAddrs, handler: impl Handler) {
        let listener = TcpListener::bind(addr).unwrap();

        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut r = RequestReader::new(&mut stream).read();
            let mut w = ResponseWriter::new_empty();
            handler.handle(&mut w, &mut r);
            let response = w.write();
            stream.write_all(&response).unwrap();
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
