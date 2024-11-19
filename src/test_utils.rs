use std::io::{self, ErrorKind, Read};

pub struct ErrReader<I> {
    it: I,
}

impl<I> ErrReader<I> {
    pub fn new(prefix: impl IntoIterator<IntoIter = I>) -> Self {
        Self {
            it: prefix.into_iter(),
        }
    }
}

impl<'a, I: Iterator<Item = &'a u8>> Read for ErrReader<I> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Some(byte) = self.it.next() {
            buf[0] = *byte;
            return Ok(1);
        }
        Err(io::Error::new(ErrorKind::Other, "error"))
    }
}
