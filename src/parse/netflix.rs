use anyhow::{Context, Error, Result};
use scraper::{Html, Selector};
use serde_json::Value;
use std::ops::Not;

use crate::{
    job::SalaryRange,
    parse::{Parse, ParseSelf, Role},
};

pub struct Netflix {}

impl Netflix {
    fn parse_title_and_team(data: &Value) -> Result<(String, Option<String>)> {
        let title_and_team = data["title"]
            .as_str()
            .map(html_escape::decode_html_entities)
            .ok_or_else(|| {
                Error::msg("failed to parse key 'title' in job posting JSON data as string")
            })?;

        Ok(match title_and_team.split_once(", ") {
            Some((title, team)) => (
                title.to_owned(),
                team.is_empty().not().then_some(team).map(|t| t.to_owned()),
            ),
            None => (title_and_team.to_string(), None),
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
        .map(|s| s.trim().to_owned())
    }

    fn parse_salary_range(data: &Value) -> Result<Option<SalaryRange>> {
        let description = data["description"]
        .as_str()
        .map(html_escape::decode_html_entities)
        .ok_or_else(|| {
            Error::msg(
                "failed to parse key 'baseSalary.value.unitText' in job posting JSON data as string",
            )
        })?;

        SalaryRange::parse(&description)
    }
}

impl Parse<&str, Role> for Netflix {
    fn parse(&self, s: &str) -> Result<Option<Role>> {
        let job_posting_data_selector =
            Selector::parse(r#"script[type="application/ld+json"]"#).unwrap();
        let document = Html::parse_document(s);
        let job_posting_data = document
            .select(&job_posting_data_selector)
            .next()
            .context("failed to select job posting data from document")?
            .text()
            .collect::<Vec<_>>()
            .join("");

        let job_posting_data: Value = serde_json::from_str(&job_posting_data)
            .context("failed to parse job posting data as JSON")?;

        let company = Self::parse_company(&job_posting_data)?;
        let (title, team) = Self::parse_title_and_team(&job_posting_data)?;
        let salary_range = Self::parse_salary_range(&job_posting_data).unwrap_or(None);

        Ok(Some(Role {
            company: company.to_owned(),
            title: title.to_owned(),
            team: team.to_owned(),
            salary_range,
        }))
    }
}
