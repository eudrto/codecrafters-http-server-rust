use std::{
    collections::HashMap,
    io::{BufRead, BufReader, ErrorKind, Read, Take},
};

use thiserror::Error;
use tracing::info;

#[derive(Debug)]
struct RequestLine<'a> {
    line: &'a str,
}

impl<'a> RequestLine<'a> {
    fn new(line: &'a str) -> Self {
        Self { line }
    }

    #[allow(unused)]
    fn http_method(&self) -> &'a str {
        self.line.split(" ").nth(0).unwrap()
    }

    fn request_target(&self) -> &'a str {
        self.line.split(" ").nth(1).unwrap()
    }

    #[allow(unused)]
    fn http_version(&self) -> &'a str {
        self.line.split(" ").nth(2).unwrap()
    }
}

#[derive(Debug)]
pub struct Request {
    request_line: String,
    param: Option<String>,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

impl Request {
    pub fn new(
        request_line: String,
        param: Option<String>,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Self {
        Self {
            request_line,
            param,
            headers,
            body,
        }
    }

    #[allow(unused)]
    pub fn get_http_method(&self) -> &str {
        RequestLine::new(&self.request_line).http_method()
    }

    pub fn get_request_target(&self) -> &str {
        RequestLine::new(&self.request_line).request_target()
    }

    #[allow(unused)]
    pub fn get_http_version(&self) -> &str {
        RequestLine::new(&self.request_line).http_version()
    }

    pub fn get_param(&self) -> Option<&str> {
        self.param.as_deref()
    }

    pub fn set_param(&mut self, param: String) {
        self.param = Some(param);
    }

    pub fn get_header(&self, key: &str) -> Option<&str> {
        self.headers.get(&key.to_lowercase()).map(|v| v.as_str())
    }

    pub fn get_body(&self) -> Option<&[u8]> {
        self.body.as_deref()
    }
}

#[derive(Error, Debug)]
#[error("end of file")]
pub struct EndOfFile;

#[derive(Error, Debug)]
#[error("invalid request")]
pub struct InvalidRequest;

pub struct RequestReader<R> {
    buf_reader: Take<BufReader<R>>,
}

impl<R: Read> RequestReader<R> {
    pub fn new(r: R) -> Self {
        Self {
            buf_reader: BufReader::new(r).take(u64::MAX),
        }
    }

    pub fn read(&mut self) -> anyhow::Result<Request> {
        let mut request_line = String::new();
        self.buf_reader.set_limit(1024);
        let n = self.buf_reader.read_line(&mut request_line)?;
        if n == 0 {
            Err(EndOfFile)?
        }
        request_line = request_line
            .strip_suffix("\r\n")
            .ok_or(InvalidRequest)?
            .to_owned();

        if request_line.split(" ").count() != 3 {
            Err(InvalidRequest)?
        }

        info!(?request_line);

        let mut headers = HashMap::new();
        self.buf_reader.set_limit(8 * 1024);
        loop {
            let mut line = String::new();
            self.buf_reader.read_line(&mut line)?;
            line = line.strip_suffix("\r\n").ok_or(InvalidRequest)?.to_owned();

            if line.is_empty() {
                break;
            }
            let (k, v) = line.split_once(":").ok_or(InvalidRequest)?;
            headers.insert(k.to_lowercase(), v.trim().to_owned());
        }

        self.buf_reader.set_limit(8 * 1024);
        let mut body = None;
        if RequestLine::new(&request_line).http_method().to_lowercase() == "post" {
            let content_length = headers.get("content-length").ok_or(InvalidRequest)?;
            let mut buf = vec![0; content_length.parse().map_err(|_| InvalidRequest)?];
            if let Err(err) = self.buf_reader.read_exact(&mut buf) {
                if err.kind() == ErrorKind::UnexpectedEof {
                    Err(InvalidRequest)?
                } else {
                    Err(err)?
                }
            }
            body = Some(buf)
        };

        Ok(Request::new(request_line, None, headers, body))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        io::{self, Cursor},
    };

    use crate::test_utils::{ErrReader, InfReader};

    use super::{EndOfFile, InvalidRequest, Request, RequestReader};

    #[test]
    fn test_request() {
        let r = Request::new("GET / HTTP/1.1".to_owned(), None, HashMap::new(), None);
        assert_eq!(r.get_http_method(), "GET");
        assert_eq!(r.get_request_target(), "/");
        assert_eq!(r.get_http_version(), "HTTP/1.1");
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // request line
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    #[test]
    fn test_request_reader_status_line_ok() {
        let cursor = Cursor::new("GET / HTTP/1.1\r\n\r\n");
        let mut request_reader = RequestReader::new(cursor);
        let r = request_reader.read().unwrap();
        assert_eq!(r.get_http_method(), "GET");
        assert_eq!(r.get_request_target(), "/");
        assert_eq!(r.get_http_version(), "HTTP/1.1");
    }

    #[test]
    fn test_request_reader_status_line_empty() {
        let cursor = Cursor::new("");
        let mut request_reader = RequestReader::new(cursor);
        let res = request_reader.read();
        res.unwrap_err().downcast_ref::<EndOfFile>().unwrap();
    }

    #[test]
    fn test_request_reader_status_line_error() {
        let err_reader = ErrReader::new(b"GET /");
        let mut request_reader = RequestReader::new(err_reader);
        let res = request_reader.read();
        res.unwrap_err().downcast_ref::<io::Error>().unwrap();
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // headers
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    #[test]
    fn test_request_reader_headers_ok() {
        let data = "GET / HTTP/1.1\r\nAccept: */*\r\n\r\n";
        let cursor = Cursor::new(data);
        let mut request_reader = RequestReader::new(cursor);
        let r = request_reader.read().unwrap();
        assert_eq!(r.get_http_method(), "GET");
        assert_eq!(r.get_request_target(), "/");
        assert_eq!(r.get_http_version(), "HTTP/1.1");
        assert_eq!(r.get_header("accept").unwrap(), "*/*");
    }

    #[test]
    fn test_request_reader_headers_no_colon() {
        let data = "GET / HTTP/1.1\r\nAccept */*\r\n\r\n";
        let cursor = Cursor::new(data);
        let mut request_reader = RequestReader::new(cursor);
        let res = request_reader.read();
        res.unwrap_err().downcast_ref::<InvalidRequest>().unwrap();
    }

    #[test]
    fn test_request_reader_headers_error() {
        let err_reader = ErrReader::new(b"GET / HTTP/1.1\r\nAccept");
        let mut request_reader = RequestReader::new(err_reader);
        let res = request_reader.read();
        res.unwrap_err().downcast_ref::<io::Error>().unwrap();
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // newline
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    #[test]
    fn test_request_reader_missing_newline_after_headers() {
        {
            let data = "GET / HTTP/1.1\r\n";
            let cursor = Cursor::new(data);
            let mut request_reader = RequestReader::new(cursor);
            let res = request_reader.read();
            res.unwrap_err().downcast_ref::<InvalidRequest>().unwrap();
        }
        {
            let data = "GET / HTTP/1.1\r\nAccept: */*\r\n";
            let cursor = Cursor::new(data);
            let mut request_reader = RequestReader::new(cursor);
            let res = request_reader.read();
            res.unwrap_err().downcast_ref::<InvalidRequest>().unwrap();
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // infinite stream
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    #[test]
    fn test_request_reader_infinite_stream() {
        {
            let prefix = b"GET / HTTP/1.1\r\n";
            let repeat = 0;
            let inf_reader = InfReader::new(prefix, repeat);
            let mut request_reader = RequestReader::new(inf_reader);
            let res = request_reader.read();
            res.unwrap_err().downcast_ref::<InvalidRequest>().unwrap();
        }
        {
            let prefix = b"GET / HTTP/1.1\r\nAccept: */*\r\n";
            let repeat = 0;
            let inf_reader = InfReader::new(prefix, repeat);
            let mut request_reader = RequestReader::new(inf_reader);
            let res = request_reader.read();
            res.unwrap_err().downcast_ref::<InvalidRequest>().unwrap();
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // multiple requests
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    #[test]
    fn test_request_reader_multiple_requests() {
        let fst = "GET / HTTP/1.1\r\n\r\n";
        let snd = "GET /about HTTP/1.1\r\n\r\n";
        let cursor = Cursor::new(format!("{}{}", fst, snd));
        let mut request_reader = RequestReader::new(cursor);

        {
            let r = request_reader.read().unwrap();
            assert_eq!(r.get_http_method(), "GET");
            assert_eq!(r.get_request_target(), "/");
            assert_eq!(r.get_http_version(), "HTTP/1.1");
        }

        {
            let r = request_reader.read().unwrap();
            assert_eq!(r.get_http_method(), "GET");
            assert_eq!(r.get_request_target(), "/about");
            assert_eq!(r.get_http_version(), "HTTP/1.1");
        }

        let res = request_reader.read();
        res.unwrap_err().downcast_ref::<EndOfFile>().unwrap();
    }
}
