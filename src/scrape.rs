use anyhow::{Context, Error, Result};
use camino::Utf8Path as Path;
use std::{
    fs::{DirBuilder, File},
    io::{Read, copy},
};
use url::Url;

use crate::job::Job;

pub mod hiring_cafe;

pub trait JobScraper {
    fn fetch(&self, url: &Url) -> Result<impl Read> {
        let url = url.as_str();
        let response = ureq::get(url).call()?;
        let reader = response.into_body().into_reader();
        Ok(reader)
    }

    fn parse(&self, reader: impl Read) -> Result<Job>;
}

pub fn scrape(job_scraper: &impl JobScraper, url: &Url, content_dir: Option<&Path>) -> Result<Job> {
    let mut content_buffer: Vec<u8> = Vec::new();
    job_scraper
        .fetch(url)
        .context(format!("failed to fetch job content at {}", url.as_str()))?
        .read_to_end(&mut content_buffer)
        .context("failed to read job content into buffer")?;

    let mut content_buffer = &content_buffer[..];

    let job = job_scraper
        .parse(content_buffer)
        .context(format!("failed to parse job content at {}", url.as_str()))?;

    if let (Some(dir), Ok(filename)) = (content_dir, job.filename()) {
        if dir.is_file() {
            return Err(Error::msg(format!(
                "content directory {} is a file, not a directory",
                dir
            )));
        }

        DirBuilder::new()
            .recursive(true)
            .create(dir)
            .context(format!(
                "failed to create content directory {} for scraped content",
                dir
            ))?;

        let filepath = dir.join(filename);
        let mut f = File::create_new(&filepath).context(format!(
            "failed to create file {} for scraped content",
            filepath
        ))?;

        copy(&mut content_buffer, &mut f).context("failed to write scraped content")?;
    }

    Ok(job)
}
