use std::io::{BufRead, BufReader, Read};

#[derive(Debug)]
struct StatusLine<'a> {
    line: &'a str,
}

impl<'a> StatusLine<'a> {
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
    status_line: String,
    param: Option<String>,
}

impl Request {
    pub fn new(status_line: String, param: Option<String>) -> Self {
        Self { status_line, param }
    }

    #[allow(unused)]
    pub fn get_http_method(&self) -> &str {
        StatusLine::new(&self.status_line).http_method()
    }

    pub fn get_request_target(&self) -> &str {
        StatusLine::new(&self.status_line).request_target()
    }

    #[allow(unused)]
    pub fn get_http_version(&self) -> &str {
        StatusLine::new(&self.status_line).http_version()
    }

    pub fn get_param(&self) -> Option<&str> {
        self.param.as_deref()
    }

    pub fn set_param(&mut self, param: String) {
        self.param = Some(param);
    }
}

pub struct RequestReader<R> {
    buf_reader: BufReader<R>,
}

impl<R: Read> RequestReader<R> {
    pub fn new(r: R) -> Self {
        Self {
            buf_reader: BufReader::new(r),
        }
    }

    pub fn read(&mut self) -> Request {
        let mut buf = String::new();
        self.buf_reader.read_line(&mut buf).unwrap();
        buf.truncate(buf.len() - 2);
        Request::new(buf, None)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::request::RequestReader;

    use super::Request;

    #[test]
    fn test_request() {
        let r = Request::new("GET / HTTP/1.1".to_owned(), None);
        assert_eq!(r.get_http_method(), "GET");
        assert_eq!(r.get_request_target(), "/");
        assert_eq!(r.get_http_version(), "HTTP/1.1");
    }

    #[test]
    fn test_request_reader() {
        let cursor = Cursor::new("GET / HTTP/1.1\r\n\r\n");
        let mut request_reader = RequestReader::new(cursor);
        let r = request_reader.read();
        assert_eq!(r.get_http_method(), "GET");
        assert_eq!(r.get_request_target(), "/");
        assert_eq!(r.get_http_version(), "HTTP/1.1");
    }
}
