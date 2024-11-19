#[cfg(test)]
use std::net::SocketAddr;
use std::{
    io::Write,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    thread,
    time::Duration,
};

use crate::{
    request::{Request, RequestReader},
    response_writer::ResponseWriter,
    status_code_registry::ReasonPhrase,
};

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
                        eprintln!("{:?}", err);
                        continue;
                    }
                };

                s.spawn(|| {
                    if let Err(err) = handle_request(stream, read_timeout, &handler) {
                        eprintln!("{}", err);
                    }
                });
            }
        });
    }
}

fn handle_request(
    mut stream: TcpStream,
    read_timeout: Option<Duration>,
    handler: &impl Handler,
) -> anyhow::Result<()> {
    stream.set_read_timeout(read_timeout)?;

    let mut r = match RequestReader::new(&mut stream).read() {
        Ok(r) => r,
        Err(err) => {
            let mut w = ResponseWriter::new_empty();
            w.set_reason_phrase(ReasonPhrase::BadRequest);
            stream.write_all(&w.write())?;
            Err(err)?
        }
    };
    dbg!(&r);

    let mut w = ResponseWriter::new_empty();
    handler.handle(&mut w, &mut r);
    let response = w.write();
    stream.write_all(&response)?;
    Ok(())
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
        io::{self, Write},
        net::{TcpListener, TcpStream},
        thread,
        time::Duration,
    };

    use super::{handle_request, noop_handler};

    #[test]
    fn test_request_reader_timeout() {
        let timeout = Some(Duration::from_millis(100));

        let listener = TcpListener::bind("localhost:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let server_handle = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle_request(stream, timeout, &noop_handler())
        });

        let _client_handle = thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            stream.write_all(b"GET / HTTP/1.1\r\n").unwrap();
            loop {}
        });

        let res = server_handle.join().unwrap();
        res.unwrap_err().downcast_ref::<io::Error>().unwrap();
    }
}
