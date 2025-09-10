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

pub enum HiringCafeScraper {
    Http(HttpScraper),
    LocalFile(LocalFileScraper),
}

pub fn new(url: &Url) -> Result<HiringCafeScraper> {
    match url.scheme() {
        "file" => Ok(HiringCafeScraper::LocalFile(LocalFileScraper {})),
        "https" => Ok(HiringCafeScraper::Http(HttpScraper {})),
        _ => Err(anyhow!(format!(
            "got unsupported URL scheme {}",
            url.scheme()
        ))),
    }
}

fn parse_title_and_team(document: &Html) -> Result<(String, String)> {
    let title_and_team_selector = Selector::parse("h2.font-extrabold").unwrap();
    let title_and_team = document
        .select(&title_and_team_selector)
        .next()
        .context("failed to select title and team from document")?
        .text()
        .to_owned()
        .collect::<Vec<_>>()
        .join("");

    let (title, team) = title_and_team
        .split_once(", ")
        .ok_or_else(|| Error::msg("failed to parse title and team from document"))?;

    Ok((title.to_owned(), team.to_owned()))
}

fn parse_company(document: &Html) -> Result<String> {
    let company_selector = Selector::parse(".text-xl").unwrap();
    Ok(document
        .select(&company_selector)
        .next()
        .context("failed to select company from document")?
        .text()
        .collect::<Vec<_>>()
        .join("")
        .replace("@ ", ""))
}

fn parse_salary_range(document: &Html) -> Result<Option<SalaryRange>> {
    let salary_selector = Selector::parse("span.rounded:nth-child(1)").unwrap();
    let salary = document
        .select(&salary_selector)
        .next()
        .context("failed to select salary range from document")?
        .text()
        .collect::<Vec<_>>()
        .join("");

    let salary_re = Regex::new(r"\$(\d+)k-\$(\d+)k\/yr").unwrap();
    let salary_range = salary_re
        .captures_iter(&salary)
        .next()
        .map(|c| -> Result<SalaryRange> {
            let (_, [lower, upper]) = c.extract();
            let lower = lower.parse::<u32>()? * 1000;
            let upper = upper.parse::<u32>()? * 1000;

            if lower > upper {
                return Err(Error::msg(format!(
                    "failed to parse salary range, lower bound {} is greater than upper bound {}",
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

impl JobScraper for HiringCafeScraper {
    fn scrape(&self, url: &Url) -> Result<ScrapedContent> {
        match self {
            HiringCafeScraper::Http(scraper) => scraper.scrape(url),
            HiringCafeScraper::LocalFile(scraper) => scraper.scrape(url),
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
        let company = parse_company(&document)?;
        let (title, team) = parse_title_and_team(&document)?;
        let salary_range = parse_salary_range(&document)?;

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
        let document = Html::parse_document(&html);
        let company = parse_company(&document)?;
        let (title, team) = parse_title_and_team(&document)?;
        let salary_range = parse_salary_range(&document)?;

        Ok(ScrapedContent {
            job: Job {
                listing_url: None,
                company,
                title: title.to_owned(),
                team: team.to_owned(),
                salary_range,
            },
            content: html,
        })
    }
}
