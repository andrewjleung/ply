use anyhow::{Context, Error, Result};
use camino::Utf8Path as Path;
use std::{
    fs::{DirBuilder, File},
    io::{Read, Write, copy},
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

    let content_buffer = &content_buffer[..];

    let job = job_scraper
        .parse(content_buffer)
        .context(format!("failed to parse job content at {}", url.as_str()))?;

    // TODO: split this out so it's not a weird little side effect
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

        let markdown_content = htmd::HtmlToMarkdown::builder()
            .skip_tags(vec!["script", "style"])
            .build()
            .convert(
                &String::from_utf8(content_buffer.to_vec())
                    .context("failed to read scraped content into string")?,
            )
            .context("failed to convert scraped HTML to markdown")?;

        f.write(markdown_content.as_bytes())
            .context("failed to write scraped markdown content")?;

        println!("scraped listing written to {}", filepath)

        // match markdown_content {
        //     Ok(content) => {
        //         f.write(content.as_bytes())
        //             .context("failed to write scraped markdown content")?;
        //     }
        //     Err(_) => {
        //         // TDOO: log matched error
        //         copy(&mut content_buffer, &mut f)
        //             .context("failed to write scraped html content")?;
        //     }
        // };
    }

    Ok(job)
}
