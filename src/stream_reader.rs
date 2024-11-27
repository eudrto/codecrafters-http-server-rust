use std::io::{self, BufRead, BufReader, Read, Take};

use thiserror::Error;

#[derive(Error, Debug)]
#[error("end of file")]
pub struct EndOfFile;

pub struct StreamReader<R> {
    buf_reader: Take<BufReader<R>>,
}

impl<R: Read> StreamReader<R> {
    pub fn new(r: R) -> Self {
        Self {
            buf_reader: BufReader::new(r).take(u64::MAX),
        }
    }

    pub fn set_limit(&mut self, limit: u64) {
        self.buf_reader.set_limit(limit);
    }

    pub fn read_line(&mut self, buf: &mut String) -> anyhow::Result<()> {
        let n = self.buf_reader.read_line(buf)?;
        if n == 0 {
            Err(EndOfFile)?
        }
        Ok(())
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.buf_reader.read_exact(buf)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fmt::{Debug, Display},
        io::Cursor,
    };

    use crate::test_utils::InfReader;

    use super::{EndOfFile, StreamReader};

    fn check_ok(input: &str, want: &str) {
        let cursor = Cursor::new(input);
        let mut stream_reader = StreamReader::new(cursor);
        let mut buf = String::new();
        stream_reader.read_line(&mut buf).unwrap();
        assert_eq!(buf, want);
    }

    fn check_err<E>(input: &str)
    where
        E: Display + Debug + Send + Sync + 'static,
    {
        let cursor = Cursor::new(input);
        let mut stream_reader = StreamReader::new(cursor);
        let res = stream_reader.read_line(&mut String::new());
        res.unwrap_err().downcast_ref::<E>().unwrap();
    }

    #[test]
    fn test_read_line_simple() {
        check_ok("Hello World!\r\n", "Hello World!\r\n");
    }

    #[test]
    fn test_read_line_only_newline() {
        check_ok("\r\n", "\r\n");
    }

    #[test]
    fn test_read_line_empty_line() {
        check_err::<EndOfFile>("");
    }

    #[test]
    fn test_read_line_no_newline() {
        check_ok("Hello World!", "Hello World!");
    }

    #[test]
    fn test_read_line_limit_exceeded() {
        let input = "foo\r\n";
        let cursor = Cursor::new(input);
        let mut stream_reader = StreamReader::new(cursor);
        let mut buf = String::new();

        stream_reader.set_limit(5);
        stream_reader.read_line(&mut buf).unwrap();
        assert_eq!(buf, "foo\r\n");

        let res = stream_reader.read_line(&mut buf);
        res.unwrap_err().downcast_ref::<EndOfFile>().unwrap();
    }

    #[test]
    fn test_stream_reader_infinite_stream() {
        let prefix = b"Hello World!";
        let repeat = 0;
        let inf_reader = InfReader::new(prefix, repeat);
        let mut stream_reader = StreamReader::new(inf_reader);

        stream_reader.set_limit(1024);
        stream_reader.read_line(&mut String::new()).unwrap();
    }
}
