use std::{collections::HashMap, fmt::Display};

use lazy_static::lazy_static;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

// https://www.iana.org/assignments/http-status-codes/http-status-codes.xhtml

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, EnumIter)]
#[repr(u16)]
pub enum ReasonPhrase {
    // Informational
    Continue = 100,
    SwitchingProtocols = 101,
    Processing = 102,
    EarlyHints = 103,
    // Success
    OK = 200,
    Created = 201,
    Accepted = 202,
    NonAuthoritativeInformation = 203,
    NoContent = 204,
    ResetContent = 205,
    PartialContent = 206,
    MultiStatus = 207,
    AlreadyReported = 208,
    IMUsed = 226,
    // Redirection
    MultipleChoices = 300,
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    UseProxy = 305,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,
    // Client Error
    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthenticationRequired = 407,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    ContentTooLarge = 413,
    URITooLong = 414,
    UnsupportedMediaType = 415,
    RangeNotSatisfiable = 416,
    ExpectationFailed = 417,
    MisdirectedRequest = 421,
    UnprocessableContent = 422,
    Locked = 423,
    FailedDependency = 424,
    TooEarly = 425,
    UpgradeRequired = 426,
    PreconditionRequired = 428,
    TooManyRequests = 429,
    RequestHeaderFieldsTooLarge = 431,
    UnavailableForLegalReasons = 451,
    // Server Error
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    HTTPVersionNotSupported = 505,
    VariantAlsoNegotiates = 506,
    InsufficientStorage = 507,
    LoopDetected = 508,
    NetworkAuthenticationRequired = 511,
}

impl ReasonPhrase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Continue => "Continue",
            Self::SwitchingProtocols => "Switching Protocols",
            Self::Processing => "Processing",
            Self::EarlyHints => "Early Hints",
            Self::OK => "OK",
            Self::Created => "Created",
            Self::Accepted => "Accepted",
            Self::NonAuthoritativeInformation => "Non-Authoritative Information",
            Self::NoContent => "No Content",
            Self::ResetContent => "Reset Content",
            Self::PartialContent => "Partial Content",
            Self::MultiStatus => "Multi-Status",
            Self::AlreadyReported => "Already Reported",
            Self::IMUsed => "IM Used",
            Self::MultipleChoices => "Multiple Choices",
            Self::MovedPermanently => "Moved Permanently",
            Self::Found => "Found",
            Self::SeeOther => "See Other",
            Self::NotModified => "Not Modified",
            Self::UseProxy => "Use Proxy",
            Self::TemporaryRedirect => "Temporary Redirect",
            Self::PermanentRedirect => "Permanent Redirect",
            Self::BadRequest => "Bad Request",
            Self::Unauthorized => "Unauthorized",
            Self::PaymentRequired => "Payment Required",
            Self::Forbidden => "Forbidden",
            Self::NotFound => "Not Found",
            Self::MethodNotAllowed => "Method Not Allowed",
            Self::NotAcceptable => "Not Acceptable",
            Self::ProxyAuthenticationRequired => "Proxy Authentication Required",
            Self::RequestTimeout => "Request Timeout",
            Self::Conflict => "Conflict",
            Self::Gone => "Gone",
            Self::LengthRequired => "Length Required",
            Self::PreconditionFailed => "Precondition Failed",
            Self::ContentTooLarge => "Content Too Large",
            Self::URITooLong => "URI Too Long",
            Self::UnsupportedMediaType => "Unsupported Media Type",
            Self::RangeNotSatisfiable => "Range Not Satisfiable",
            Self::ExpectationFailed => "Expectation Failed",
            Self::MisdirectedRequest => "Misdirected Request",
            Self::UnprocessableContent => "Unprocessable Content",
            Self::Locked => "Locked",
            Self::FailedDependency => "Failed Dependency",
            Self::TooEarly => "Too Early",
            Self::UpgradeRequired => "Upgrade Required",
            Self::PreconditionRequired => "Precondition Required",
            Self::TooManyRequests => "Too Many Requests",
            Self::RequestHeaderFieldsTooLarge => "Request Header Fields Too Large",
            Self::UnavailableForLegalReasons => "Unavailable For Legal Reasons",
            Self::InternalServerError => "Internal Server Error",
            Self::NotImplemented => "Not Implemented",
            Self::BadGateway => "Bad Gateway",
            Self::ServiceUnavailable => "Service Unavailable",
            Self::GatewayTimeout => "Gateway Timeout",
            Self::HTTPVersionNotSupported => "HTTP Version Not Supported",
            Self::VariantAlsoNegotiates => "Variant Also Negotiates",
            Self::InsufficientStorage => "Insufficient Storage",
            Self::LoopDetected => "Loop Detected",
            Self::NetworkAuthenticationRequired => "Network Authentication Required",
        }
    }
}

impl Display for ReasonPhrase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

lazy_static! {
    static ref STATUS_CODE_REGISTRY: StatusCodeRegistry = StatusCodeRegistry::new();
}

pub fn get_reason_phrase(status_code: u16) -> Option<ReasonPhrase> {
    STATUS_CODE_REGISTRY.get_reason_phrase(status_code)
}

pub fn get_status_code(reason_phrase: ReasonPhrase) -> u16 {
    STATUS_CODE_REGISTRY.get_status_code(reason_phrase)
}

struct StatusCodeRegistry {
    reason_phrase_lookup: HashMap<u16, ReasonPhrase>,
    status_code_lookup: HashMap<ReasonPhrase, u16>,
}

impl StatusCodeRegistry {
    fn new() -> Self {
        let mut reason_phrase_lookup = HashMap::new();
        let mut status_code_lookup = HashMap::new();
        for reason_phrase in ReasonPhrase::iter() {
            let status_code = reason_phrase as u16;
            reason_phrase_lookup.insert(status_code, reason_phrase);
            status_code_lookup.insert(reason_phrase, status_code);
        }
        Self {
            reason_phrase_lookup,
            status_code_lookup,
        }
    }

    fn get_reason_phrase(&self, status_code: u16) -> Option<ReasonPhrase> {
        self.reason_phrase_lookup.get(&status_code).map(|x| *x)
    }

    fn get_status_code(&self, reason_phrase: ReasonPhrase) -> u16 {
        self.status_code_lookup[&reason_phrase]
    }
}

#[cfg(test)]
mod tests {
    use super::{get_reason_phrase, get_status_code, ReasonPhrase};

    #[test]
    fn test_get_reason_phrase_some() {
        let status_code = 200;
        let reason_phrase = get_reason_phrase(status_code).unwrap();
        assert_eq!(reason_phrase, ReasonPhrase::OK);
    }

    #[test]
    #[should_panic]
    fn test_get_reason_phrase_none() {
        let status_code = 600;
        get_reason_phrase(status_code).unwrap();
    }

    #[test]
    fn test_get_status_code() {
        let reason_phrase = ReasonPhrase::OK;
        let status_code = get_status_code(reason_phrase);
        assert_eq!(status_code, 200);
    }

    #[test]
    fn test_as_str() {
        let reason_phrase = ReasonPhrase::NotFound;
        assert_eq!(reason_phrase.as_str(), "Not Found");
    }
}
