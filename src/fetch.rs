use anyhow::{Context, Error, Result, anyhow};
use std::fs::read_to_string;
use url::Url;

pub enum Source {
    Http(HttpSource),
    LocalFile(LocalFileSource),
}

impl TryFrom<&Url> for Source {
    fn try_from(value: &Url) -> std::result::Result<Self, Self::Error> {
        match value.scheme() {
            "https" => Ok(Source::Http(HttpSource {
                url: value.to_owned(),
            })),
            "file" => Ok(Source::LocalFile(LocalFileSource {
                url: value.to_owned(),
            })),
            scheme => Err(anyhow!("failed to determine source for URL sceme {scheme}")),
        }
    }

    type Error = Error;
}

impl Fetch for Source {
    fn fetch(&self) -> Result<String> {
        match self {
            Source::Http(s) => s.fetch(),
            Source::LocalFile(s) => s.fetch(),
        }
    }
}

pub struct HttpSource {
    url: Url,
}

impl TryFrom<&Url> for HttpSource {
    fn try_from(value: &Url) -> std::result::Result<Self, Self::Error> {
        if value.scheme() != "https" {
            return Err(anyhow!(
                "incorrect URL scheme, expected https but got {}",
                value.scheme()
            ));
        }

        Ok(HttpSource {
            url: value.to_owned(),
        })
    }

    type Error = Error;
}

pub struct LocalFileSource {
    url: Url,
}

impl TryFrom<&Url> for LocalFileSource {
    fn try_from(value: &Url) -> std::result::Result<Self, Self::Error> {
        if value.scheme() != "file" {
            return Err(anyhow!(
                "incorrect URL scheme, expected file but got {}",
                value.scheme()
            ));
        }

        Ok(LocalFileSource {
            url: value.to_owned(),
        })
    }

    type Error = Error;
}

pub trait Fetch {
    fn fetch(&self) -> Result<String>;
}

impl Fetch for HttpSource {
    fn fetch(&self) -> Result<String> {
        ureq::get(self.url.as_str())
            .call()?
            .body_mut()
            .read_to_string()
            .context("failed to read HTTP response to string")
            .context(format!(
                "failed to fetch source at {} with HTTPS",
                self.url.as_str()
            ))
    }
}

impl Fetch for LocalFileSource {
    fn fetch(&self) -> Result<String> {
        let path = self
            .url
            .to_file_path()
            .map_err(|()| Error::msg("failed to convert local URL scrape target to file path"))?;

        read_to_string(&path).context(format!("failed to read file at {}", path.to_string_lossy()))
    }
}
