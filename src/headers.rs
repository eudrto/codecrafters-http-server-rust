use anyhow::anyhow;

use crate::multi_map::{MultiMap, Value};

#[derive(Debug)]
pub struct Headers<'a>(MultiMap<&'a str, &'a str>);

impl<'a> Headers<'a> {
    pub fn new(mm: MultiMap<&'a str, &'a str>) -> Self {
        Self(mm)
    }

    #[allow(unused)]
    pub fn new_empty() -> Self {
        Self(MultiMap::new_empty())
    }

    fn parse_values_line(values_line: &str) -> Value<&str> {
        let mut it = values_line.split(',');
        let first = it.next().unwrap();

        match it.next() {
            None => Value::Scalar(first.trim()),
            Some(second) => Value::Vector(
                [first, second]
                    .into_iter()
                    .chain(it)
                    .map(|value| value.trim())
                    .collect(),
            ),
        }
    }

    pub fn parse(raw: &'a str) -> anyhow::Result<Self> {
        let mm = raw
            .lines()
            .take_while(|line| !line.is_empty())
            .map(|line| {
                let (k, values_line) = line
                    .split_once(":")
                    .ok_or(anyhow!("missing colon delimiter"))?;
                Ok((k, Self::parse_values_line(values_line)))
            })
            .collect::<Result<_, anyhow::Error>>()?;

        Ok(Self::new(mm))
    }

    pub fn get_scalar(&self, key: &str) -> anyhow::Result<Option<&str>> {
        Ok(self.0.get_scalar(key.to_lowercase().as_str())?.copied())
    }

    pub fn get_iter(&self, key: &str) -> Option<impl Iterator<Item = &str> + '_> {
        self.0
            .get_value_iter(key.to_lowercase().as_str())
            .map(|it| it.copied())
    }

    pub fn get_accept_encoding(&self) -> Option<impl Iterator<Item = &str> + '_> {
        self.get_iter("accept-encoding")
    }

    pub fn get_connection(&self) -> Option<impl Iterator<Item = &str> + '_> {
        self.get_iter("connection")
    }

    pub fn get_content_length(&self) -> anyhow::Result<Option<usize>> {
        match self
            .get_scalar("content-length")?
            .map(|length| length.parse::<usize>())
        {
            Some(Ok(length)) => Ok(Some(length)),
            Some(Err(err)) => Err(err)?,
            None => Ok(None),
        }
    }

    pub fn get_user_agent(&self) -> anyhow::Result<Option<&str>> {
        self.get_scalar("user-agent")
    }
}

#[cfg(test)]
mod tests {
    use crate::headers::Headers;

    #[test]
    fn test_parse_simple() {
        let raw = "accept: */*\r\n\r\n";
        let headers = Headers::parse(raw).unwrap();
        assert_eq!(headers.get_scalar("accept").unwrap().unwrap(), "*/*");
    }

    #[test]
    fn test_parse_comma_separated() {
        let raw = "accept: text/html, application/json\r\n\r\n";
        let headers = Headers::parse(raw).unwrap();
        assert_eq!(
            headers.get_iter("Accept").unwrap().collect::<Vec<_>>(),
            vec!["text/html", "application/json"]
        );
    }

    #[test]
    fn test_parse_repeated() {
        let raw = "set-cookie: foo\r\nset-cookie: bar\r\n\r\n";
        let headers = Headers::parse(raw).unwrap();
        assert_eq!(
            headers.get_iter("Set-Cookie").unwrap().collect::<Vec<_>>(),
            vec!["foo", "bar"]
        );
    }

    #[test]
    fn test_parse_no_colon() {
        let raw = "Accept */*\r\n\r\n";
        let res = Headers::parse(raw);
        res.unwrap_err();
    }
}
