use anyhow::{Context, Error, Result};
use camino::Utf8Path as Path;
use camino::Utf8PathBuf as PathBuf;
use std::{
    fs::{DirBuilder, File},
    io::Write,
};
use url::Url;

use crate::job::Job;

pub mod hiring_cafe;

pub struct Scrape {
    pub job: Job,
    pub content: String,
}

pub trait JobScraper {
    fn scrape(&self, url: &Url) -> Result<Scrape>;
}

pub fn snapshot_content(content: &mut str, content_dir: &Path, filename: &str) -> Result<PathBuf> {
    if content_dir.is_file() {
        return Err(Error::msg(format!(
            "content directory {} is a file, not a directory",
            content_dir
        )));
    }

    DirBuilder::new()
        .recursive(true)
        .create(content_dir)
        .context(format!(
            "failed to create content directory {} for scraped content",
            content_dir
        ))?;

    let filepath = content_dir.join(filename);
    let mut f = File::create_new(&filepath).context(format!(
        "failed to create file {} for scraped content",
        filepath
    ))?;

    let markdown_content = htmd::HtmlToMarkdown::builder()
        .skip_tags(vec!["script", "style"])
        .build()
        .convert(content)
        .context("failed to convert scraped HTML to markdown")?;

    f.write(markdown_content.as_bytes())
        .context("failed to write scraped markdown content")?;

    Ok(filepath)
}
