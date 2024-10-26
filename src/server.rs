use std::{
    io::Write,
    net::{TcpListener, ToSocketAddrs},
};

use crate::response_writer::ResponseWriter;

#[derive(Debug)]
pub struct Server;

impl Server {
    pub fn run(addr: impl ToSocketAddrs, handler: impl Fn(&mut ResponseWriter)) {
        let listener = TcpListener::bind(addr).unwrap();

        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut w = ResponseWriter::new_empty();
            handler(&mut w);
            let response = w.write();
            stream.write_all(&response).unwrap();
        }
    }
}
