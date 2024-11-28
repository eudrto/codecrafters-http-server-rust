use std::io::{ErrorKind, Read};

use thiserror::Error;
use tracing::info;

use crate::{headers::Headers, multi_map::MultiMap, stream_reader::StreamReader};

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
    headers: Headers,
    body: Option<Vec<u8>>,
}

impl Request {
    pub fn new(
        request_line: String,
        param: Option<String>,
        headers: Headers,
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

    pub fn get_headers(&self) -> &Headers {
        &self.headers
    }

    pub fn get_body(&self) -> Option<&[u8]> {
        self.body.as_deref()
    }
}

#[derive(Error, Debug)]
#[error("invalid request")]
pub struct InvalidRequest;

pub struct RequestReader<R> {
    stream_reader: StreamReader<R>,
}

impl<R: Read> RequestReader<R> {
    pub fn new(r: R) -> Self {
        Self {
            stream_reader: StreamReader::new(r),
        }
    }

    pub fn read(&mut self, buf: &mut String) -> anyhow::Result<Request> {
        self.stream_reader.set_limit(1024);
        self.stream_reader.read_line(buf)?;

        let request_line = buf.strip_suffix("\r\n").ok_or(InvalidRequest)?.to_owned();

        if request_line.split(" ").count() != 3 {
            Err(InvalidRequest)?
        }

        info!(?request_line);

        let mut mm = MultiMap::new_empty();
        self.stream_reader.set_limit(8 * 1024);
        loop {
            buf.clear();
            self.stream_reader.read_line(buf)?;
            let line = buf.strip_suffix("\r\n").ok_or(InvalidRequest)?.to_owned();

            if line.is_empty() {
                break;
            }
            let (k, values_line) = line.split_once(":").ok_or(InvalidRequest)?;
            let values = values_line
                .split(",")
                .map(|v| v.trim().to_owned())
                .collect();
            mm.insert_vector(k.to_lowercase(), values);
        }
        let headers = Headers::new(mm);

        self.stream_reader.set_limit(8 * 1024);
        let mut body = None;
        if RequestLine::new(&request_line).http_method().to_lowercase() == "post" {
            let content_length = headers
                .get_content_length()
                .map_err(|_| InvalidRequest)?
                .ok_or(InvalidRequest)?;
            let mut buf = vec![0; content_length];
            if let Err(err) = self.stream_reader.read_exact(&mut buf) {
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
    use std::io::{self, Cursor};

    use crate::{headers::Headers, stream_reader::EndOfFile, test_utils::ErrReader};

    use super::{InvalidRequest, Request, RequestReader};

    #[test]
    fn test_request() {
        let r = Request::new(
            "GET / HTTP/1.1".to_owned(),
            None,
            Headers::new_empty(),
            None,
        );
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
        let mut buf = String::new();
        let r = request_reader.read(&mut buf).unwrap();
        assert_eq!(r.get_http_method(), "GET");
        assert_eq!(r.get_request_target(), "/");
        assert_eq!(r.get_http_version(), "HTTP/1.1");
    }

    #[test]
    fn test_request_reader_status_line_empty() {
        let cursor = Cursor::new("");
        let mut request_reader = RequestReader::new(cursor);
        let res = request_reader.read(&mut String::new());
        res.unwrap_err().downcast_ref::<EndOfFile>().unwrap();
    }

    #[test]
    fn test_request_reader_status_line_error() {
        let err_reader = ErrReader::new(b"GET /");
        let mut request_reader = RequestReader::new(err_reader);
        let res = request_reader.read(&mut String::new());
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
        let mut buf = String::new();
        let r = request_reader.read(&mut buf).unwrap();
        assert_eq!(r.get_http_method(), "GET");
        assert_eq!(r.get_request_target(), "/");
        assert_eq!(r.get_http_version(), "HTTP/1.1");
        assert_eq!(
            r.get_headers().get_scalar("accept").unwrap().unwrap(),
            "*/*"
        );
    }

    #[test]
    fn test_request_reader_comma_separated_headers_ok() {
        let data = "GET / HTTP/1.1\r\nAccept: text/html, application/json\r\n\r\n";
        let cursor = Cursor::new(data);
        let mut request_reader = RequestReader::new(cursor);
        let mut buf = String::new();
        let r = request_reader.read(&mut buf).unwrap();
        assert_eq!(r.get_http_method(), "GET");
        assert_eq!(r.get_request_target(), "/");
        assert_eq!(r.get_http_version(), "HTTP/1.1");
        assert_eq!(
            r.get_headers()
                .get_iter("accept")
                .unwrap()
                .collect::<Vec<_>>(),
            vec!["text/html", "application/json"]
        );
    }

    #[test]
    fn test_request_reader_repeated_headers_ok() {
        let data = "GET / HTTP/1.1\r\nSet-Cookie: foo\r\nSet-Cookie: bar\r\n\r\n";
        let cursor = Cursor::new(data);
        let mut request_reader = RequestReader::new(cursor);
        let mut buf = String::new();
        let r = request_reader.read(&mut buf).unwrap();
        assert_eq!(r.get_http_method(), "GET");
        assert_eq!(r.get_request_target(), "/");
        assert_eq!(r.get_http_version(), "HTTP/1.1");
        assert_eq!(
            r.get_headers()
                .get_iter("Set-Cookie")
                .unwrap()
                .collect::<Vec<_>>(),
            vec!["foo", "bar"]
        );
    }

    #[test]
    fn test_request_reader_headers_no_colon() {
        let data = "GET / HTTP/1.1\r\nAccept */*\r\n\r\n";
        let cursor = Cursor::new(data);
        let mut request_reader = RequestReader::new(cursor);
        let res = request_reader.read(&mut String::new());
        res.unwrap_err().downcast_ref::<InvalidRequest>().unwrap();
    }

    #[test]
    fn test_request_reader_headers_error() {
        let err_reader = ErrReader::new(b"GET / HTTP/1.1\r\nAccept");
        let mut request_reader = RequestReader::new(err_reader);
        let res = request_reader.read(&mut String::new());
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
            let res = request_reader.read(&mut String::new());
            res.unwrap_err().downcast_ref::<EndOfFile>().unwrap();
        }
        {
            let data = "GET / HTTP/1.1\r\nAccept: */*\r\n";
            let cursor = Cursor::new(data);
            let mut request_reader = RequestReader::new(cursor);
            let res = request_reader.read(&mut String::new());
            res.unwrap_err().downcast_ref::<EndOfFile>().unwrap();
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
            let mut buf = String::new();
            let r = request_reader.read(&mut buf).unwrap();
            assert_eq!(r.get_http_method(), "GET");
            assert_eq!(r.get_request_target(), "/");
            assert_eq!(r.get_http_version(), "HTTP/1.1");
        }

        {
            let mut buf = String::new();
            let r = request_reader.read(&mut buf).unwrap();
            assert_eq!(r.get_http_method(), "GET");
            assert_eq!(r.get_request_target(), "/about");
            assert_eq!(r.get_http_version(), "HTTP/1.1");
        }

        let res = request_reader.read(&mut String::new());
        res.unwrap_err().downcast_ref::<EndOfFile>().unwrap();
    }
}
