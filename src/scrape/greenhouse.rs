use anyhow::{Context, Error, Result, anyhow};
use regex::Regex;
use scraper::{Html, Selector};
use std::fs::read_to_string;
use url::Url;

use crate::{
    job::{Job, SalaryRange},
    scrape::{JobScraper, ScrapedContent},
};

pub struct HttpScraper {}
pub struct LocalFileScraper {}

pub enum GreenhouseScraper {
    Http(HttpScraper),
    LocalFile(LocalFileScraper),
}

pub fn new(url: &Url) -> Result<GreenhouseScraper> {
    match url.scheme() {
        "file" => Ok(GreenhouseScraper::LocalFile(LocalFileScraper {})),
        "https" => Ok(GreenhouseScraper::Http(HttpScraper {})),
        _ => Err(anyhow!(format!(
            "got unsupported URL scheme {}",
            url.scheme()
        ))),
    }
}

fn parse_company_title_and_team(document: &Html) -> Result<(String, String, Option<String>)> {
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

    let dash_re = Regex::new(
        r"^Job Application for (?P<title>[^-()]+) - (?P<team>[^();\r\n]+)(?:\s*\([^)]*\))? +at +(?P<company>.+)$"
    ).unwrap();

    let delim_re = Regex::new(
        r"^Job Application for (?P<title>[^-()]+)[,:] (?P<team>[^();\r\n]+)(?:\s*\([^)]*\))? +at +(?P<company>.+)$"
    ).unwrap();

    let paren_re = Regex::new(
        r"^Job Application for (?P<title>[^()]+) \((?P<team>[^;()]+)(?:;[^)]*)?\) +at +(?P<company>.+)$"
    ).unwrap();

    let no_team_re =
        Regex::new(r"^Job Application for (?P<title>.+) +at +(?P<company>.+)$").unwrap();

    if let Some(caps) = dash_re.captures(&document_title) {
        let title = caps.name("title").unwrap().as_str().trim().to_string();
        let team = caps.name("team").map(|m| m.as_str().trim().to_string());
        let company = caps.name("company").unwrap().as_str().trim().to_string();
        return Ok((company, title, team));
    }

    if let Some(caps) = delim_re.captures(&document_title) {
        let title = caps.name("title").unwrap().as_str().trim().to_string();
        let team = caps.name("team").map(|m| m.as_str().trim().to_string());
        let company = caps.name("company").unwrap().as_str().trim().to_string();
        return Ok((company, title, team));
    }

    if let Some(caps) = paren_re.captures(&document_title) {
        let title = caps.name("title").unwrap().as_str().trim().to_string();
        let team = caps.name("team").map(|m| m.as_str().trim().to_string());
        let company = caps.name("company").unwrap().as_str().trim().to_string();
        return Ok((company, title, team));
    }

    if let Some(caps) = no_team_re.captures(&document_title) {
        let title = caps.name("title").unwrap().as_str().trim().to_string();
        let company = caps.name("company").unwrap().as_str().trim().to_string();
        return Ok((company, title, None));
    }

    Err(anyhow!("failed to match title {document_title}"))
}

fn parse_salary_range(html: &str) -> Result<Option<SalaryRange>> {
    let salary_re = Regex::new(r"\$((?:\d+,\d{3})|(?:\d+k)).*\$((?:\d+,\d{3})|(?:\d+k))")
        .expect("failed to compile salary range regex");
    let salary_range = salary_re
        // TODO: from string to html back to string... bad
        .captures_iter(html)
        .next()
        .map(|c| -> Result<SalaryRange> {
            let (_, [lower, upper]) = c.extract();
            let lower = lower.replace(",", "").replace("k", "000").parse::<u32>()?;
            let upper = upper.replace(",", "").replace("k", "000").parse::<u32>()?;

            if lower > upper {
                return Err(Error::msg(format!(
                    "lower bound {} is greater than upper bound {}",
                    lower, upper
                )));
            }

            Ok(SalaryRange {
                lower,
                range: Some(upper.abs_diff(lower)),
            })
        });

    Ok(match salary_range {
        Some(Ok(range)) => Some(range),
        Some(Err(error)) => return Err(error),
        None => None,
    })
}

impl JobScraper for GreenhouseScraper {
    fn scrape(&self, url: &Url) -> Result<ScrapedContent> {
        match self {
            GreenhouseScraper::Http(scraper) => scraper.scrape(url),
            GreenhouseScraper::LocalFile(scraper) => scraper.scrape(url),
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
        let (company, title, team) = parse_company_title_and_team(&document)
            .context("failed to parse company, title, and team")?;
        let salary_range = parse_salary_range(&html).context("failed to parse salary range")?;

        Ok(ScrapedContent {
            job: Job {
                listing_url: Some(url.to_owned()),
                company,
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
            "failed to read greenhouse listing at {}",
            path.to_string_lossy()
        ))?;
        let document = Html::parse_document(&html);
        let (company, title, team) = parse_company_title_and_team(&document)
            .context("failed to parse company, title, and team")?;
        let salary_range = parse_salary_range(&html).context("failed to parse salary range")?;

        Ok(ScrapedContent {
            job: Job {
                listing_url: Some(url.to_owned()),
                company,
                title: title.to_owned(),
                team: team.to_owned(),
                salary_range,
            },
            content: html,
        })
    }
}
