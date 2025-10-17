use anyhow::{Context, Error, Result, anyhow};
use regex::Regex;
use scraper::{Html, Selector};
use std::fs::read_to_string;
use url::Url;

use crate::{
    job::{Job, SalaryRange},
    parse::Parse,
    scrape::{JobScraper, ScrapedContent},
};

pub struct HttpScraper {}
pub struct LocalFileScraper {}

pub enum GoogleScraper {
    Http(HttpScraper),
    LocalFile(LocalFileScraper),
}

pub fn new(url: &Url) -> Result<GoogleScraper> {
    match url.scheme() {
        "file" => Ok(GoogleScraper::LocalFile(LocalFileScraper {})),
        "https" => Ok(GoogleScraper::Http(HttpScraper {})),
        _ => Err(anyhow!(format!(
            "got unsupported URL scheme {}",
            url.scheme()
        ))),
    }
}

fn parse_title_and_team(document: &Html) -> Result<(String, Option<String>)> {
    let document_title_selector =
        Selector::parse("head > title").expect("failed to compile title selector");
    let document_title = &document
        .select(&document_title_selector)
        .next()
        .context("failed to select document title from document")?
        .text()
        .collect::<Vec<_>>()
        .join("");
    let document_title = html_escape::decode_html_entities(document_title);
    let title_re = Regex::new(r"^(?P<title>.*), (?P<team>.*) — Google Careers$").unwrap();

    if let Some(caps) = title_re.captures(&document_title) {
        let title = caps.name("title").unwrap().as_str().trim().to_string();
        let team = caps.name("team").map(|m| m.as_str().trim().to_string());
        return Ok((title, team));
    }

    Err(anyhow!("failed to match title {document_title}"))
}

impl JobScraper for GoogleScraper {
    fn scrape(&self, url: &Url) -> Result<ScrapedContent> {
        match self {
            GoogleScraper::Http(scraper) => scraper.scrape(url),
            GoogleScraper::LocalFile(scraper) => scraper.scrape(url),
        }
    }
}

impl JobScraper for HttpScraper {
    fn scrape(&self, url: &Url) -> Result<ScrapedContent> {
        let html = ureq::get(url.as_str())
            .call()?
            .body_mut()
            .read_to_string()
            .context("failed to read scraped HTTP response to string")?;

        let document = Html::parse_document(&html);
        let (title, team) =
            parse_title_and_team(&document).context("failed to parse title and team")?;
        let salary_range = SalaryRange::parse(&html)?;

        Ok(ScrapedContent {
            job: Job {
                listing_url: Some(url.to_owned()),
                company: "Google".to_string(),
                title: title.to_owned(),
                team: team.to_owned(),
                salary_range,
            },
            content: html,
        })
    }
}

impl JobScraper for LocalFileScraper {
    fn scrape(&self, url: &Url) -> Result<ScrapedContent> {
        let path = url
            .to_file_path()
            .map_err(|()| Error::msg("failed to convert local URL scrape target to file path"))?;

        let html = read_to_string(&path).context(format!(
            "failed to read google listing at {}",
            path.to_string_lossy()
        ))?;
        let document = Html::parse_document(&html);
        let (title, team) =
            parse_title_and_team(&document).context("failed to parse title and team")?;
        let salary_range = SalaryRange::parse(&html)?;

        Ok(ScrapedContent {
            job: Job {
                listing_url: Some(url.to_owned()),
                company: "Google".to_string(),
                title: title.to_owned(),
                team: team.to_owned(),
                salary_range,
            },
            content: html,
        })
    }
}
