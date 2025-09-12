use anyhow::{Context, Error, Result, anyhow};
use scraper::{Html, Selector};
use serde_json::Value;
use std::{fs::read_to_string, ops::Not};
use url::Url;

use crate::{
    job::{Job, SalaryRange},
    scrape::{JobScraper, ScrapedContent},
};

pub struct HttpScraper {}
pub struct LocalFileScraper {}

pub enum AshbyHqScraper {
    Http(HttpScraper),
    LocalFile(LocalFileScraper),
}

pub fn new(url: &Url) -> Result<AshbyHqScraper> {
    match url.scheme() {
        "file" => Ok(AshbyHqScraper::LocalFile(LocalFileScraper {})),
        "https" => Ok(AshbyHqScraper::Http(HttpScraper {})),
        _ => Err(anyhow!(format!(
            "got unsupported URL scheme {}",
            url.scheme()
        ))),
    }
}

fn parse_title_and_team(data: &Value) -> Result<(String, Option<String>)> {
    let title_and_team = data["title"].as_str().ok_or_else(|| {
        Error::msg("failed to parse key 'title' in job posting JSON data as string")
    })?;

    Ok(match title_and_team.split_once(", ") {
        Some((title, team)) => (
            title.to_owned(),
            team.is_empty().not().then_some(team).map(|t| t.to_owned()),
        ),
        None => (title_and_team.to_owned(), None),
    })
}

fn parse_company(data: &Value) -> Result<String> {
    data["hiringOrganization"]["name"]
        .as_str()
        .ok_or_else(|| {
            Error::msg(
                "failed to parse key 'hiringOrganization.name' in job posting JSON data as string",
            )
        })
        .map(|s| s.to_owned())
}

fn parse_salary_range(data: &Value) -> Result<Option<SalaryRange>> {
    let unit = data["baseSalary"]["value"]["unitText"]
        .as_str()
        .ok_or_else(|| {
            Error::msg(
                "failed to parse key 'baseSalary.value.unitText' in job posting JSON data as string",
            )
        })?;

    if unit != "YEAR" {
        return Err(anyhow!("salary range unit is not yearly, got {unit}"));
    }

    let min = data["baseSalary"]["value"]["minValue"]
        .as_u64().ok_or_else(|| anyhow!("failed to parse key 'baseSalary.value.minValue into u64"))
        .and_then(|u| {
            u32::try_from(u).context(format!("failed to parse value {u} key 'baseSalary.value.minValue' in job posting JSON data as u32")
)
        })?;

    let max = data["baseSalary"]["value"]["maxValue"]
        .as_u64().ok_or_else(|| anyhow!("failed to parse key 'baseSalary.value.maxValue into u64"))
        .and_then(|u| {
            u32::try_from(u).context(format!("failed to parse value {u} key 'baseSalary.value.maxValue' in job posting JSON data as u32")
)
        })?;

    Ok(Some(SalaryRange {
        lower: min,
        range: Some(max.abs_diff(min)),
    }))
}

impl JobScraper for AshbyHqScraper {
    fn scrape(&self, url: &Url) -> Result<ScrapedContent> {
        match self {
            AshbyHqScraper::Http(scraper) => scraper.scrape(url),
            AshbyHqScraper::LocalFile(scraper) => scraper.scrape(url),
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

        let job_posting_data_selector =
            Selector::parse(r#"script[type="application/ld+json"]"#).unwrap();
        let document = Html::parse_document(&html);
        let job_posting_data = document
            .select(&job_posting_data_selector)
            .next()
            .context("failed to select job posting data from document")?
            .text()
            .collect::<Vec<_>>()
            .join("");

        let job_posting_data: Value = serde_json::from_str(&job_posting_data)
            .context("failed to parse job posting data as JSON")?;

        let company = parse_company(&job_posting_data)?;
        let (title, team) = parse_title_and_team(&job_posting_data)?;
        let salary_range = parse_salary_range(&job_posting_data)?;

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
            "failed to read hiringcafe listing at {}",
            path.to_string_lossy()
        ))?;
        let job_posting_data_selector =
            Selector::parse(r#"script[type="application/ld+json"]"#).unwrap();
        let document = Html::parse_document(&html);
        let job_posting_data = document
            .select(&job_posting_data_selector)
            .next()
            .context("failed to select job posting data from document")?
            .text()
            .collect::<Vec<_>>()
            .join("");

        let job_posting_data: Value = serde_json::from_str(&job_posting_data)
            .context("failed to parse job posting data as JSON")?;

        let company = parse_company(&job_posting_data)?;
        let (title, team) = parse_title_and_team(&job_posting_data)?;
        let salary_range = parse_salary_range(&job_posting_data)?;

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
