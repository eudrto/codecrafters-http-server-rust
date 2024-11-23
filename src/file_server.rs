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

pub fn new_file_writer(base_path: impl Into<PathBuf>) -> impl Handler {
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

        if let Some(parent) = path.parent() {
            if fs::create_dir_all(parent).is_err() {
                w.set_reason_phrase(ReasonPhrase::InternalServerError);
                return;
            }
        }

        if let Err(err) = fs::write(path, r.get_body().unwrap()) {
            error!("{}", err);
            w.set_reason_phrase(ReasonPhrase::InternalServerError);
            return;
        }

        w.set_reason_phrase(ReasonPhrase::Created);
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

    use reqwest::blocking::Client;
    use tempdir::TempDir;

    use crate::{
        router::Router,
        server::{HttpMethod, Server},
    };

    use super::{build_path, new_file_retriever, new_file_writer};

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
            router.add_route(HttpMethod::Get, "/files/", &file_retriever);
            server.run(router);
        });

        let url = format!("http://{}/files/hello", addr);
        let resp = reqwest::blocking::get(url).unwrap();
        let body = resp.text().unwrap();
        assert_eq!(body, "Hello World!");

        // TODO: Make sure the temp dir is removed even if the test fails
        fs::remove_dir_all(&*base_path).unwrap();
    }

    #[test]
    fn test_file_writer() {
        let server = Server::new("localhost:0");
        let addr = server.local_addr();

        let tmp_dir = TempDir::new("").unwrap();
        let base_path = Arc::new(tmp_dir.into_path());
        let clone = Arc::clone(&base_path);
        thread::spawn(move || {
            let mut router = Router::new();
            let file_writer = new_file_writer(&*clone);
            router.add_route(HttpMethod::Post, "/files/", &file_writer);
            server.run(router);
        });

        let client = Client::new();
        let url = format!("http://{}/files/hello", addr);
        let resp = client.post(url).body("Hello World!").send().unwrap();
        assert_eq!(resp.status(), 201);

        let contents = fs::read_to_string(base_path.join("hello")).unwrap();
        assert_eq!(contents, "Hello World!");

        // TODO: Make sure the temp dir is removed even if the test fails
        fs::remove_dir_all(&*base_path).unwrap();
    }

    #[test]
    fn test_concurrent_writes() {
        let server = Server::new("localhost:0");
        let addr = server.local_addr();

        let tmp_dir = TempDir::new("").unwrap();
        let base_path = Arc::new(tmp_dir.into_path());
        let clone = Arc::clone(&base_path);
        thread::spawn(move || {
            let mut router = Router::new();
            let file_writer = new_file_writer(&*clone);
            router.add_route(HttpMethod::Post, "/files/", &file_writer);
            server.run(router);
        });

        let handles: Vec<_> = (0..5)
            .map(|i| {
                thread::spawn(move || {
                    let client = Client::new();
                    let url = format!("http://{}/files/hello", addr);
                    let resp = client.post(url).body(vec![i; 4 * 1024]).send().unwrap();
                    assert_eq!(resp.status(), 201);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let contents = fs::read(base_path.join("hello")).unwrap();
        assert_eq!(contents.len(), 4 * 1024);

        assert!(contents.iter().all(|x| *x == contents[0]));
        assert!((0..5).collect::<Vec<_>>().contains(&contents[0]));

        // TODO: Make sure the temp dir is removed even if the test fails
        fs::remove_dir_all(&*base_path).unwrap();
    }
}
