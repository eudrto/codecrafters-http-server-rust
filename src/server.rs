#[cfg(test)]
use std::net::SocketAddr;
use std::{
    io::Write,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    thread,
    time::Duration,
};

use strum_macros::Display;
use tracing::{error, info, span, Level, Span};

use crate::{
    request::{EndOfFile, Request, RequestReader},
    response_writer::ResponseWriter,
    status_code_registry::ReasonPhrase,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display)]
pub enum HttpMethod {
    Get = 0,
    Post = 1,
}

impl TryFrom<usize> for HttpMethod {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(HttpMethod::Get),
            1 => Ok(HttpMethod::Post),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for HttpMethod {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "get" => Ok(HttpMethod::Get),
            "post" => Ok(HttpMethod::Post),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn new(addr: impl ToSocketAddrs) -> Self {
        Self {
            listener: TcpListener::bind(addr).unwrap(),
        }
    }

    #[cfg(test)]
    pub fn local_addr(&self) -> SocketAddr {
        self.listener.local_addr().unwrap()
    }

    pub fn run(&self, handler: impl Handler + Sync) {
        let read_timeout = Some(Duration::from_secs(10));
        thread::scope(|s| {
            for stream in self.listener.incoming() {
                let stream = match stream {
                    Ok(stream) => stream,
                    Err(err) => {
                        error!(?err);
                        continue;
                    }
                };

                s.spawn(|| {
                    let span = create_conn_span(&stream);
                    let _guard = span.enter();
                    info!("new conn");

                    if let Err(err) = handle_connection(stream, read_timeout, &handler) {
                        error!(?err);
                    }

                    info!("conn end");
                });
            }
        });
    }
}

#[derive(Debug)]
enum ConnCtrl {
    KeepAlive,
    Close,
}

fn handle_connection(
    stream: TcpStream,
    read_timeout: Option<Duration>,
    handler: &impl Handler,
) -> anyhow::Result<()> {
    let (reader, writer) = (&stream, &stream);
    reader.set_read_timeout(read_timeout)?;
    let mut request_reader = RequestReader::new(reader);

    loop {
        match handle_request(&mut request_reader, writer, handler) {
            Ok(ConnCtrl::KeepAlive) => continue,
            Ok(ConnCtrl::Close) => return Ok(()),
            Err(err) => {
                return Err(err);
            }
        }
    }
}

fn handle_request(
    request_reader: &mut RequestReader<&TcpStream>,
    mut writer: &TcpStream,
    handler: &impl Handler,
) -> anyhow::Result<ConnCtrl> {
    let mut r = match request_reader.read() {
        Ok(r) => r,
        Err(err) => {
            if err.downcast_ref::<EndOfFile>().is_some() {
                return Ok(ConnCtrl::Close);
            }

            error!(?err);
            let mut w = ResponseWriter::new_empty();
            w.set_reason_phrase(ReasonPhrase::BadRequest);
            writer.write_all(&w.write())?;
            return Ok(ConnCtrl::Close);
        }
    };

    let span = create_req_span(&r);
    let _guard = span.enter();
    info!(?r);

    let conn_ctrl = match r
        .get_headers()
        .get_connection()
        .map(|mut it| it.any(|val| val == "close"))
    {
        Some(true) => ConnCtrl::Close,
        _ => ConnCtrl::KeepAlive,
    };

    let mut w = ResponseWriter::new_empty();
    handler.handle(&mut w, &mut r);
    let response = w.write();
    writer.write_all(&response)?;
    Ok(conn_ctrl)
}

fn create_conn_span(stream: &TcpStream) -> Span {
    let peer_addr = match stream.peer_addr() {
        Ok(addr) => &addr.to_string(),
        Err(err) => {
            error!(?err);
            "unknown"
        }
    };

    span!(Level::INFO, "conn", peer_addr)
}

fn create_req_span(r: &Request) -> Span {
    let http_method = r.get_http_method();
    let request_target = r.get_request_target();
    span!(
        Level::INFO,
        "req",
        method = http_method,
        target = request_target
    )
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

#[cfg(test)]
pub fn noop_handler() -> impl Handler {
    |_: &mut ResponseWriter, _: &mut Request| {}
}

#[cfg(test)]
pub mod tests {
    use std::{
        io::{BufReader, Read, Write},
        net::{TcpListener, TcpStream},
        thread,
        time::Duration,
    };

    use crate::{
        request::Request, response_writer::ResponseWriter, status_code_registry::ReasonPhrase,
    };

    use super::{handle_connection, noop_handler, Server};

    #[test]
    fn test_request_reader_timeout() {
        let timeout = Some(Duration::from_millis(100));

        let listener = TcpListener::bind("localhost:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let server_handle = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle_connection(stream, timeout, &noop_handler())
        });

        let _client_handle = thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            stream.write_all(b"GET / HTTP/1.1\r\n").unwrap();
            loop {}
        });

        server_handle.join().unwrap().unwrap();
    }

    #[test]
    fn test_persistent_connection() {
        let timeout = Some(Duration::from_millis(100));

        let server = Server::new("localhost:0");
        let addr = server.local_addr();

        thread::spawn(move || {
            server.run(|w: &mut ResponseWriter, _: &mut Request| {
                w.set_reason_phrase(ReasonPhrase::OK);
            });
        });

        let stream = TcpStream::connect(addr).unwrap();
        let (r, mut writer) = (&stream, &stream);
        r.set_read_timeout(timeout).unwrap();
        let mut reader = BufReader::new(r);

        writer.write_all(b"GET / HTTP/1.1\r\n\r\n").unwrap();

        // Read to ends tries to read until EOF but cannot do so
        // because the connection is not closed.
        // Instead, the timeout expires and an error is returned.
        let mut buf = vec![];
        let res = reader.read_to_end(&mut buf);
        res.unwrap_err();
    }
}
