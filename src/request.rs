use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
};

use thiserror::Error;

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
}

impl Request {
    pub fn new(
        request_line: String,
        param: Option<String>,
        headers: HashMap<String, String>,
    ) -> Self {
        Self {
            request_line,
            param,
            headers,
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

    pub fn get_header(&self, key: &str) -> &str {
        &self.headers[&key.to_lowercase()]
    }
}

#[derive(Error, Debug)]
#[error("invalid request")]
pub struct InvalidRequest;

pub struct RequestReader<R> {
    buf_reader: BufReader<R>,
}

impl<R: Read> RequestReader<R> {
    pub fn new(r: R) -> Self {
        Self {
            buf_reader: BufReader::new(r),
        }
    }

    pub fn read(self) -> anyhow::Result<Request> {
        let mut it = self.buf_reader.lines();

        let request_line = it.next().unwrap()?;
        if request_line.split(" ").count() != 3 {
            Err(InvalidRequest)?
        }

        let mut headers = HashMap::new();
        while let Some(line) = it.next() {
            let line = line?;
            if line.is_empty() {
                break;
            }
            let (k, v) = line.split_once(":").ok_or(InvalidRequest)?;
            headers.insert(k.to_lowercase(), v.trim().to_owned());
        }

        Ok(Request::new(request_line, None, headers))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        io::{self, Cursor},
    };

    use crate::test_utils::ErrReader;

    use super::{InvalidRequest, Request, RequestReader};

    #[test]
    fn test_request() {
        let r = Request::new("GET / HTTP/1.1".to_owned(), None, HashMap::new());
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
        let request_reader = RequestReader::new(cursor);
        let r = request_reader.read().unwrap();
        assert_eq!(r.get_http_method(), "GET");
        assert_eq!(r.get_request_target(), "/");
        assert_eq!(r.get_http_version(), "HTTP/1.1");
    }

    #[test]
    fn test_request_reader_status_line_empty() {
        let cursor = Cursor::new("");
        let request_reader = RequestReader::new(cursor);
        let res = request_reader.read();
        res.unwrap_err().downcast_ref::<InvalidRequest>().unwrap();
    }

    #[test]
    fn test_request_reader_status_line_error() {
        let err_reader = ErrReader::new(b"GET /");
        let request_reader = RequestReader::new(err_reader);
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
        let request_reader = RequestReader::new(cursor);
        let r = request_reader.read().unwrap();
        assert_eq!(r.get_http_method(), "GET");
        assert_eq!(r.get_request_target(), "/");
        assert_eq!(r.get_http_version(), "HTTP/1.1");
        assert_eq!(r.get_header("accept"), "*/*");
    }

    #[test]
    fn test_request_reader_headers_no_colon() {
        let data = "GET / HTTP/1.1\r\nAccept */*\r\n\r\n";
        let cursor = Cursor::new(data);
        let request_reader = RequestReader::new(cursor);
        let res = request_reader.read();
        res.unwrap_err().downcast_ref::<InvalidRequest>().unwrap();
    }

    #[test]
    fn test_request_reader_headers_error() {
        let err_reader = ErrReader::new(b"GET / HTTP/1.1\r\nAccept");
        let request_reader = RequestReader::new(err_reader);
        let res = request_reader.read();
        res.unwrap_err().downcast_ref::<io::Error>().unwrap();
    }
}
