use anyhow::{Context, Error, Result, anyhow};
use scraper::{Html, Selector};
use serde_json::Value;
use std::ops::Not;

use crate::{
    job::SalaryRange,
    parse::Role,
    parse::{Parse, salary::parse_yearly_bound},
};

pub struct Ashby {}

impl Ashby {
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

    pub fn parse_company(data: &Value) -> Result<String> {
        data["hiringOrganization"]["name"]
        .as_str()
        .ok_or_else(|| {
            Error::msg(
                "failed to parse key 'hiringOrganization.name' in job posting JSON data as string",
            )
        })
        .map(|s| s.trim().to_owned())
    }

    pub fn parse_salary_range(data: &Value) -> Result<Option<SalaryRange>> {
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

        let lower = data["baseSalary"]["value"]["minValue"]
            .as_str()
            .map(|v| parse_yearly_bound(v, "year"))
            .transpose()
            .context("failed to parse lower bound")?;

        let upper = data["baseSalary"]["value"]["maxValue"]
            .as_str()
            .map(|v| parse_yearly_bound(v, "year"))
            .transpose()
            .context("failed to parse upper bound")?;

        SalaryRange::try_from_maybe_bounds(lower, upper)
    }
}

impl Parse<&str, Role> for Ashby {
    fn parse(&self, s: &str) -> Result<Option<Role>> {
        let document = Html::parse_document(s);
        let job_posting_data_selector =
            Selector::parse(r#"script[type="application/ld+json"]"#).unwrap();

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
        let salary_range = Self::parse_salary_range(&job_posting_data)?;

        Ok(Some(Role {
            company,
            title: title.to_owned(),
            team: team.to_owned(),
            salary_range,
        }))
    }
}
