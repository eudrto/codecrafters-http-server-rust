use std::{
    io::{self, ErrorKind, Read},
    iter::{self, Chain, Repeat},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// ErrfReader
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// InfReader
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

pub struct InfReader {
    it: Chain<Box<dyn Iterator<Item = u8>>, Repeat<u8>>,
}

impl InfReader {
    pub fn new<'a, I: Iterator<Item = &'a u8>>(
        prefix: impl IntoIterator<IntoIter = I>,
        repeat: u8,
    ) -> Self {
        let prefix: Box<dyn Iterator<Item = u8>> =
            Box::new(prefix.into_iter().cloned().collect::<Vec<_>>().into_iter());
        Self {
            it: prefix.chain(iter::repeat(repeat)),
        }
    }
}

impl Read for InfReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let data: Vec<_> = self.it.by_ref().take(buf.len()).collect();
        buf[..data.len()].copy_from_slice(&data);
        Ok(data.len())
    }
}
