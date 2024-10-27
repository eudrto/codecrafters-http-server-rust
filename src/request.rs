use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
};

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

pub struct RequestReader<R> {
    buf_reader: BufReader<R>,
}

impl<R: Read> RequestReader<R> {
    pub fn new(r: R) -> Self {
        Self {
            buf_reader: BufReader::new(r),
        }
    }

    pub fn read(self) -> Request {
        let mut it = self.buf_reader.lines();
        let request_line = it.next().unwrap().unwrap();

        let headers = it
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .map(|line| {
                let (k, v) = line.split_once(":").unwrap();
                (k.to_lowercase(), v.trim().to_lowercase())
            })
            .collect();

        Request::new(request_line, None, headers)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, io::Cursor};

    use crate::request::RequestReader;

    use super::Request;

    #[test]
    fn test_request() {
        let r = Request::new("GET / HTTP/1.1".to_owned(), None, HashMap::new());
        assert_eq!(r.get_http_method(), "GET");
        assert_eq!(r.get_request_target(), "/");
        assert_eq!(r.get_http_version(), "HTTP/1.1");
    }

    #[test]
    fn test_request_reader() {
        let cursor = Cursor::new("GET / HTTP/1.1\r\n\r\n");
        let request_reader = RequestReader::new(cursor);
        let r = request_reader.read();
        assert_eq!(r.get_http_method(), "GET");
        assert_eq!(r.get_request_target(), "/");
        assert_eq!(r.get_http_version(), "HTTP/1.1");
    }
}
