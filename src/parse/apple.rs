use anyhow::{Context, Error, Result, anyhow};
use scraper::{Html, Selector};
use serde_json::Value;
use std::ops::Not;

use crate::{parse::Parse, parse::Role};

pub struct Apple {}

impl Apple {
    fn parse_title_and_team(data: &Value) -> Result<(String, Option<String>)> {
        let title_and_team = data["loaderData"]["jobDetails"]["jobsData"]["postingTitle"].as_str().ok_or_else(|| {
            Error::msg("failed to parse key 'loaderData.jobDetails.jobsData.postingTitle' in job posting JSON data as string")
        })?;

        Ok(match title_and_team.split_once(" - ") {
            Some((title, team)) => (
                title.to_owned(),
                team.is_empty().not().then_some(team).map(|t| t.to_owned()),
            ),
            None => (title_and_team.to_owned(), None),
        })
    }
}

impl Parse<&str, Role> for Apple {
    fn parse(&self, s: &str) -> Result<Option<Role>> {
        let document = Html::parse_document(s);

        let job_posting_data_selector = Selector::parse(r#"#root > script"#).unwrap();

        let job_posting_data = document
            .select(&job_posting_data_selector)
            .next()
            .context("failed to select job posting data from document")?
            .text()
            .collect::<Vec<_>>()
            .join("");

        let job_posting_data = job_posting_data
            .strip_prefix(r#"window.__staticRouterHydrationData = JSON.parse(""#)
            .ok_or(anyhow!("failed to strip prefix from hydration data"))?
            .strip_suffix(r#"");"#)
            .ok_or(anyhow!("failed to strip suffix from hydration data"))?
            .replace(r#"\\"#, r#"\"#)
            .replace(r#"\""#, r#"""#);

        let job_posting_data: Value = serde_json::from_str(&job_posting_data)
            .context("failed to parse job posting data as JSON")?;

        let (title, team) = Self::parse_title_and_team(&job_posting_data)?;

        Ok(Some(Role {
            company: String::from("Apple"),
            title: title.to_owned(),
            team: team.to_owned(),
            salary_range: None,
        }))
    }
}
