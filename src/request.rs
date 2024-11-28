use std::io::{ErrorKind, Read};

use anyhow::anyhow;
use thiserror::Error;

use crate::{headers::Headers, slice_ext, stream_reader::StreamReader};

#[derive(Debug)]
pub struct RequestLine<'a> {
    http_method: &'a str,
    request_target: &'a str,
    http_version: &'a str,
}

impl<'a> RequestLine<'a> {
    fn new(http_method: &'a str, request_target: &'a str, http_version: &'a str) -> Self {
        Self {
            http_method,
            request_target,
            http_version,
        }
    }

    pub fn parse(raw: &'a str) -> anyhow::Result<Self> {
        let raw = raw.strip_suffix("\r\n").ok_or(anyhow!("missing crlf"))?;
        let mut it = raw.split(|c| c == ' ');
        let http_method = it.next().ok_or(anyhow!("empty http method"))?;
        let request_target = it.next().ok_or(anyhow!("empty request target"))?;
        let http_version = it.next().ok_or(anyhow!("empty http version"))?;
        Ok(Self::new(http_method, request_target, http_version))
    }

    #[allow(unused)]
    pub fn http_method(&self) -> &'a str {
        self.http_method
    }

    pub fn request_target(&self) -> &'a str {
        self.request_target
    }

    #[allow(unused)]
    pub fn http_version(&self) -> &'a str {
        self.http_version
    }
}

#[derive(Debug)]
pub struct Request<'a> {
    request_line: RequestLine<'a>,
    param: Option<&'a str>,
    headers: Headers<'a>,
    body: Option<Vec<u8>>,
}

impl<'a> Request<'a> {
    pub fn new(
        request_line: RequestLine<'a>,
        param: Option<&'a str>,
        headers: Headers<'a>,
        body: Option<Vec<u8>>,
    ) -> Self {
        Self {
            request_line,
            param,
            headers,
            body,
        }
    }

    pub fn get_http_method(&self) -> &'a str {
        self.request_line.http_method()
    }

    pub fn get_request_target(&self) -> &'a str {
        self.request_line.request_target()
    }

    #[allow(unused)]
    pub fn get_http_version(&self) -> &'a str {
        self.request_line.http_version()
    }

    pub fn get_param(&self) -> Option<&str> {
        self.param.as_deref()
    }

    pub fn set_param(&mut self, param: &'a str) {
        self.param = Some(param);
    }

    pub fn get_headers(&self) -> &Headers {
        &self.headers
    }

    pub fn get_body(&self) -> Option<&[u8]> {
        self.body.as_deref()
    }
}

pub fn make_keys_lowercase(bytes: &mut [u8]) {
    let lines = slice_ext::split_pattern_mut(bytes, b"\r\n");

    for line in lines {
        let Some((k, _)) = slice_ext::split_once_mut(line, b":") else {
            return;
        };
        k.make_ascii_lowercase();
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

    pub fn read_metadata<'a>(&mut self, buf: &'a mut String) -> anyhow::Result<usize> {
        self.stream_reader.set_limit(1024);
        self.stream_reader.read_line(buf)?;
        if !buf.ends_with("\r\n") {
            Err(InvalidRequest)?
        }
        let request_line_end = buf.len();

        self.stream_reader.set_limit(8 * 1024);
        let mut start = request_line_end;
        loop {
            self.stream_reader.read_line(buf)?;
            let mut line = &buf[start..];
            line = line.strip_suffix("\r\n").ok_or(InvalidRequest)?;
            if line.is_empty() {
                return Ok(request_line_end);
            }
            start = buf.len();
        }
    }

    pub fn read_body<'a>(
        &mut self,
        request_line: &RequestLine,
        headers: &Headers,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        self.stream_reader.set_limit(8 * 1024);
        let mut body = None;
        if request_line.http_method().to_lowercase() == "post" {
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
        return Ok(body);
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Cursor};

    use crate::{request::RequestLine, stream_reader::EndOfFile, test_utils::ErrReader};

    use super::RequestReader;

    #[test]
    fn test_request_line_parse() {
        let request_line = RequestLine::parse("GET / HTTP/1.1\r\n").unwrap();
        assert_eq!(request_line.http_method(), "GET");
        assert_eq!(request_line.request_target(), "/");
        assert_eq!(request_line.http_version(), "HTTP/1.1");
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // request line
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    #[test]
    fn test_request_reader_status_line_ok() {
        let cursor = Cursor::new("GET / HTTP/1.1\r\n\r\n");
        let mut request_reader = RequestReader::new(cursor);
        let mut buf = String::new();
        let request_line_end = request_reader.read_metadata(&mut buf).unwrap();
        let request_line = RequestLine::parse(&buf[..request_line_end]).unwrap();
        assert_eq!(request_line.http_method(), "GET");
        assert_eq!(request_line.request_target(), "/");
        assert_eq!(request_line.http_version(), "HTTP/1.1");
    }

    #[test]
    fn test_request_reader_status_line_empty() {
        let cursor = Cursor::new("");
        let mut request_reader = RequestReader::new(cursor);
        let mut buf = String::new();
        let res = request_reader.read_metadata(&mut buf);
        res.unwrap_err().downcast_ref::<EndOfFile>().unwrap();
    }

    #[test]
    fn test_request_reader_status_line_error() {
        let err_reader = ErrReader::new(b"GET /");
        let mut request_reader = RequestReader::new(err_reader);
        let mut buf = String::new();
        let res = request_reader.read_metadata(&mut buf);
        res.unwrap_err().downcast_ref::<io::Error>().unwrap();
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // headers
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    #[test]
    fn test_request_reader_headers_error() {
        let err_reader = ErrReader::new(b"GET / HTTP/1.1\r\nAccept");
        let mut request_reader = RequestReader::new(err_reader);
        let mut buf = String::new();
        let res = request_reader.read_metadata(&mut buf);
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
            let mut buf = String::new();
            let res = request_reader.read_metadata(&mut buf);
            res.unwrap_err().downcast_ref::<EndOfFile>().unwrap();
        }
        {
            let data = "GET / HTTP/1.1\r\nAccept: */*\r\n";
            let cursor = Cursor::new(data);
            let mut request_reader = RequestReader::new(cursor);
            let mut buf = String::new();
            let res = request_reader.read_metadata(&mut buf);
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
            let request_line_end = request_reader.read_metadata(&mut buf).unwrap();
            let request_line = RequestLine::parse(&buf[..request_line_end]).unwrap();
            assert_eq!(request_line.http_method(), "GET");
            assert_eq!(request_line.request_target(), "/");
            assert_eq!(request_line.http_version(), "HTTP/1.1");
        }

        {
            let mut buf = String::new();
            let request_line_end = request_reader.read_metadata(&mut buf).unwrap();
            let request_line = RequestLine::parse(&buf[..request_line_end]).unwrap();
            assert_eq!(request_line.http_method(), "GET");
            assert_eq!(request_line.request_target(), "/about");
            assert_eq!(request_line.http_version(), "HTTP/1.1");
        }

        let mut buf = String::new();
        let res = request_reader.read_metadata(&mut buf);
        res.unwrap_err().downcast_ref::<EndOfFile>().unwrap();
    }
}
