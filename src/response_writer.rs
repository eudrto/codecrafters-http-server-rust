use crate::status_code_registry::{self, ReasonPhrase};

#[derive(Debug)]
pub struct ResponseWriter {
    status_code: Option<u16>,
    reason_phrase: Option<String>,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl ResponseWriter {
    fn new(status_code: Option<u16>, reason_phrase: Option<String>) -> Self {
        Self {
            status_code,
            reason_phrase,
            headers: vec![],
            body: vec![],
        }
    }

    pub fn new_empty() -> Self {
        Self::new(None, None)
    }

    #[allow(unused)]
    pub fn get_status_code(&self) -> Option<u16> {
        self.status_code
    }

    #[allow(unused)]
    pub fn set_status_code(&mut self, status_code: u16) {
        self.status_code = Some(status_code);
        self.reason_phrase =
            status_code_registry::get_reason_phrase(status_code).map(|r| r.to_string());
    }

    pub fn set_reason_phrase(&mut self, reason_phrase: ReasonPhrase) {
        self.status_code = Some(status_code_registry::get_status_code(reason_phrase));
        self.reason_phrase = Some(reason_phrase.to_string());
    }

    #[allow(unused)]
    pub fn set_status_line(&mut self, status_code: u16, reason_phrase: String) {
        self.status_code = Some(status_code);
        self.reason_phrase = Some(reason_phrase);
    }

    fn add_header(&mut self, k: String, v: String) {
        self.headers.push((k, v));
    }

    fn add_content_type_header(&mut self) {
        self.add_header("Content-Type".to_owned(), "text/plain".to_owned());
    }

    fn add_content_length_header(&mut self) {
        self.add_header("Content-Length".to_owned(), self.body.len().to_string());
    }

    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = body;
    }

    pub fn set_body_str(&mut self, body: &str) {
        self.set_body(body.bytes().collect());
    }

    pub fn write(mut self) -> Vec<u8> {
        let status_code = self.status_code.unwrap();
        let mut status_line = format!("HTTP/1.1 {}", status_code);
        if let Some(reason_phrase) = &self.reason_phrase {
            status_line = format!("{} {}", status_line, reason_phrase);
        }
        status_line.push_str("\r\n");

        if status_code == 404 {
            status_line.push_str("\r\n");
            return status_line.bytes().collect();
        }

        self.add_content_type_header();
        self.add_content_length_header();

        let mut headers = self
            .headers
            .into_iter()
            .map(|(k, v)| format!("{}: {}\r\n", k, v))
            .collect::<Vec<_>>()
            .join("");
        headers.push_str("\r\n");

        let mut resp = vec![];
        resp.extend(status_line.bytes());
        resp.extend(headers.bytes());
        resp.extend(self.body);
        resp
    }
}
