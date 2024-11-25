use std::io::Read;

use flate2::{bufread::GzEncoder, Compression};
use tracing::error;

use crate::{request::Request, response_writer::ResponseWriter, server::Handler};

pub fn new(handler: impl Handler) -> impl Handler {
    move |w: &mut ResponseWriter, r: &mut Request| {
        handler.handle(w, r);

        let body = w.get_body();
        if body.len() == 0 {
            return;
        }

        let Some(content_type) = w.get_content_type_header() else {
            error!("Content-Type is supposed to be present");
            return;
        };
        let content_type = String::from(content_type);

        if let Some(mut encodings) = r.get_headers().get_accept_encoding() {
            if encodings.any(|encoding| encoding == "gzip") {
                let mut gz = GzEncoder::new(body, Compression::fast());
                let mut buffer = vec![];
                if let Err(err) = gz.read_to_end(&mut buffer) {
                    eprintln!("{}", err);
                    return;
                }

                w.set_body(buffer, &content_type);
                w.add_content_encoding_header("gzip");
            }
        }
    }
}
