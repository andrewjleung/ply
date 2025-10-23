use std::ops::Not;

use anyhow::{Context, Result};
use scraper::{Html, Selector};

use crate::{job::SalaryRange, job_listing::Role, parse::Parse};

pub struct HiringCafe {}

impl HiringCafe {
    fn parse_title_and_team(document: &Html) -> Result<(String, Option<String>)> {
        let title_and_team_selector = Selector::parse("h2.font-extrabold").unwrap();
        let title_and_team = document
            .select(&title_and_team_selector)
            .next()
            .context("failed to select title and team from document")?
            .text()
            .to_owned()
            .collect::<Vec<_>>()
            .join("");

        Ok(match title_and_team.split_once(", ") {
            Some((title, team)) => (
                title.to_owned(),
                team.is_empty().not().then_some(team).map(|t| t.to_owned()),
            ),
            None => (title_and_team, None),
        })
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

        SalaryRange::parse(&salary)
    }
}

impl Parse<&str, Role> for HiringCafe {
    fn parse(s: &str) -> Result<Option<Role>> {
        let document = Html::parse_document(s);
        let company = Self::parse_company(&document)?;
        let (title, team) = Self::parse_title_and_team(&document)?;
        let salary_range = Self::parse_salary_range(&document)?;

        Ok(Some(Role {
            company: company.to_owned(),
            title: title.to_owned(),
            team: team.to_owned(),
            salary_range,
        }))
    }
}
