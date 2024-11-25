use crate::multi_map::MultiMap;

#[derive(Debug)]
pub struct Headers(MultiMap<String, String>);

impl Headers {
    pub fn new(mm: MultiMap<String, String>) -> Self {
        Self(mm)
    }

    #[cfg(test)]
    pub fn new_empty() -> Self {
        Self(MultiMap::new_empty())
    }

    pub fn get_scalar(&self, key: &str) -> anyhow::Result<Option<&str>> {
        Ok(self
            .0
            .get_scalar(key.to_lowercase().as_str())?
            .map(|s| s.as_str()))
    }

    pub fn get_iter(&self, key: &str) -> Option<impl Iterator<Item = &str> + '_> {
        self.0
            .get_value_iter(key.to_lowercase().as_str())
            .map(|it| it.map(|e| e.as_str()))
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
