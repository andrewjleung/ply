use anyhow::{Context, Result};
use std::io::Read;
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

pub fn scrape(job_scraper: &impl JobScraper, url: &Url) -> Result<Job> {
    let content = job_scraper
        .fetch(url)
        .context(format!("failed to fetch job content at {}", url.as_str()))?;

    job_scraper
        .parse(content)
        .context(format!("failed to parse job content at {}", url.as_str()))
}
