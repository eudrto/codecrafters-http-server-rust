use crate::status_code_registry::{self, ReasonPhrase};

#[derive(Debug)]
pub struct ResponseWriter {
    status_code: Option<u16>,
    reason_phrase: Option<String>,
}

impl ResponseWriter {
    fn new(status_code: Option<u16>, reason_phrase: Option<String>) -> Self {
        Self {
            status_code,
            reason_phrase,
        }
    }

    pub fn new_empty() -> Self {
        Self::new(None, None)
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

    pub fn write(self) -> Vec<u8> {
        let status_code = self.status_code.unwrap();
        let mut status_line = format!("HTTP/1.1 {}", status_code);
        if let Some(reason_phrase) = self.reason_phrase {
            status_line = format!("{} {}", status_line, reason_phrase);
        }
        status_line.push_str("\r\n");

        let mut resp = vec![];
        resp.extend(status_line.bytes());
        resp.extend_from_slice("\r\n".as_bytes());
        resp
    }
}
