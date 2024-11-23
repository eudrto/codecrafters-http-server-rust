use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use thiserror::Error;
use tracing::{error, info, warn};

use crate::{
    request::Request, response_writer::ResponseWriter, server::Handler,
    status_code_registry::ReasonPhrase,
};

pub fn new_file_retriever(base_path: impl Into<PathBuf>) -> impl Handler {
    let base_path = base_path.into();
    move |w: &mut ResponseWriter, r: &mut Request| {
        let Some(suffix) = r.get_param() else {
            w.set_reason_phrase(ReasonPhrase::BadRequest);
            return;
        };

        let Ok(path) = build_path(&base_path, suffix) else {
            w.set_reason_phrase(ReasonPhrase::BadRequest);
            return;
        };
        info!("file path: {:?}", path);

        match fs::read(path) {
            Ok(contents) => {
                w.set_reason_phrase(ReasonPhrase::OK);
                w.set_body(contents, "application/octet-stream");
            }
            Err(err) if err.kind() == ErrorKind::NotFound => {
                w.set_reason_phrase(ReasonPhrase::NotFound);
            }
            Err(err) => {
                error!("{:?}", err);
                w.set_reason_phrase(ReasonPhrase::InternalServerError);
            }
        }
    }
}

#[derive(Error, Debug)]
#[error("invalid path")]
struct InvalidPath;

fn build_path(
    base_path: impl AsRef<Path>,
    suffix: impl AsRef<Path>,
) -> Result<PathBuf, InvalidPath> {
    let path = path_clean::clean(base_path.as_ref().join(&suffix));

    let stripped = if let Ok(stripped) = base_path.as_ref().strip_prefix("./") {
        stripped
    } else {
        base_path.as_ref()
    };

    if !path.starts_with(stripped) {
        warn!("file path: {:?}", path);
        return Err(InvalidPath);
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io::Write,
        sync::Arc,
        thread,
    };

    use tempdir::TempDir;

    use crate::{file_server::new_file_retriever, router::Router, server::Server};

    use super::build_path;

    #[test]
    fn test_build_path_ok() {
        let tests = [
            ("assets", "foo/bar", "assets/foo/bar"),
            ("assets", "./foo/bar", "assets/foo/bar"),
            ("./assets", "foo/bar", "assets/foo/bar"),
            ("./assets", "./foo/bar", "assets/foo/bar"),
            ("/assets", "foo/bar", "/assets/foo/bar"),
            ("/assets", "./foo/bar", "/assets/foo/bar"),
            ("/", "foo/bar", "/foo/bar"),
            ("/", "./foo/bar", "/foo/bar"),
        ];

        for (base, suffix, want) in tests {
            assert_eq!(build_path(base, suffix).unwrap().as_os_str(), want);
        }
    }

    #[test]
    fn test_build_path_err() {
        let tests = [
            ("assets", "../secrets"),
            ("./assets", "../secrets"),
            ("/assets", "/secrets"),
        ];

        for (base, suffix) in tests {
            assert!(build_path(base, suffix).is_err());
        }
    }

    #[test]
    fn test_file_retriever() {
        let server = Server::new("localhost:0");
        let addr = server.local_addr();

        let tmp_dir = TempDir::new("").unwrap();
        let file_path = tmp_dir.path().join("hello");
        let mut tmp_file = File::create(file_path).unwrap();
        write!(tmp_file, "Hello World!").unwrap();

        let base_path = Arc::new(tmp_dir.into_path());
        let clone = Arc::clone(&base_path);
        thread::spawn(move || {
            let mut router = Router::new();
            let file_retriever = new_file_retriever(&*clone);
            router.add_route("/files/", &file_retriever);
            server.run(router);
        });

        let url = format!("http://{}/files/hello", addr);
        let resp = reqwest::blocking::get(url).unwrap();
        let body = resp.text().unwrap();
        assert_eq!(body, "Hello World!");
        fs::remove_dir_all(&*base_path).unwrap();
    }
}
