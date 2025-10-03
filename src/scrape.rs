use anyhow::anyhow;
use anyhow::{Context, Error, Result};
use camino::Utf8Path as Path;
use camino::Utf8PathBuf as PathBuf;
use clap::ValueEnum;
use std::{
    fs::{DirBuilder, File},
    io::Write,
};
use url::Url;

use crate::job::Job;

pub mod ashbyhq;
pub mod google;
pub mod greenhouse;
pub mod hiring_cafe;
pub mod html;
pub mod meta;

pub struct ScrapedContent {
    pub job: Job,
    pub content: String,
}

pub trait JobScraper {
    fn scrape(&self, url: &Url) -> Result<ScrapedContent>;
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum JobScraperKind {
    Greenhouse,
    HiringCafe,
    AshbyHQ,
    Meta,
    Google,
}

impl JobScraper for JobScraperKind {
    // TODO: this API is weird... why do we need to supply URL twice?
    fn scrape(&self, url: &Url) -> Result<ScrapedContent> {
        match self {
            JobScraperKind::Greenhouse => greenhouse::new(url)
                .context("failed to create greenhouse scraper")?
                .scrape(url)
                .context("failed to scrape with greenhouse scraper"),
            JobScraperKind::HiringCafe => hiring_cafe::new(url)
                .context("failed to create hiring cafe scraper")?
                .scrape(url)
                .context("failed to scrape with hiring cafe scraper"),
            JobScraperKind::AshbyHQ => ashbyhq::new(url)
                .context("failed to create ashbyhq scraper")?
                .scrape(url)
                .context("failed to scrape with ashbyhq scraper"),
            JobScraperKind::Meta => meta::new(url)
                .context("failed to create meta scraper")?
                .scrape(url)
                .context("failed to scrape with meta scraper"),
            JobScraperKind::Google => google::new(url)
                .context("failed to create google scraper")?
                .scrape(url)
                .context("failed to scrape with google scraper"),
        }
    }
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

    if filepath.try_exists().context(format!(
        "failed to determine if snapshotted content already exists at {}",
        filepath
    ))? {
        return Ok(filepath);
    };

    let mut f = File::create_new(&filepath).context(format!(
        "failed to create file {} for scraped content",
        filepath
    ))?;

    let markdown_content = htmd::HtmlToMarkdown::builder()
        .skip_tags(vec!["style"])
        .build()
        .convert(content)
        .context("failed to convert scraped HTML to markdown")?;

    f.write(markdown_content.as_bytes())
        .context("failed to write scraped markdown content")?;

    Ok(filepath)
}

fn infer_scraper_kind(
    url: Url,
    scraper_kind: Option<JobScraperKind>,
) -> Result<(Url, JobScraperKind)> {
    let scraper_kind = match (url.scheme(), scraper_kind) {
        ("https", None) => match url.domain() {
            Some("hiring.cafe") => JobScraperKind::HiringCafe,
            Some("jobs.ashbyhq.com") => JobScraperKind::AshbyHQ,
            Some("job-boards.greenhouse.io") => JobScraperKind::Greenhouse,
            Some("www.metacareers.com") => JobScraperKind::Meta,
            Some("www.google.com") => JobScraperKind::Google,
            Some(domain) => {
                return Err(anyhow!(
                    "could not infer scraper kind from unrecognized HTTPS domain {domain}"
                ));
            }
            None => {
                return Err(anyhow!(
                    "could not infer scraper kind from domain-less HTTPS URL {url}"
                ));
            }
        },
        (_, None) => {
            return Err(anyhow!(
                "cannot infer scraper kind from URL scheme {}, please specify a scraper",
                url.scheme()
            ));
        }
        (_, Some(kind)) => kind,
    };

    Ok((url, scraper_kind))
}

pub fn scrape(url: &Url, scraper_kind: Option<JobScraperKind>) -> Result<ScrapedContent> {
    let (url, scraper_kind) = infer_scraper_kind(url.to_owned(), scraper_kind)?;
    scraper_kind.scrape(&url)
}
