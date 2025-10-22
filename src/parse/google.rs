use anyhow::{Context, Result, anyhow};
use regex::Regex;
use scraper::{Html, Selector};

use crate::{job::SalaryRange, job_listing::Role, parse::Parse};

pub struct Google {}

impl Google {
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
}

impl Parse<&str, Role> for Google {
    fn parse(s: &str) -> Result<Option<Role>> {
        let document = Html::parse_document(s);
        let (title, team) =
            Google::parse_title_and_team(&document).context("failed to parse title and team")?;

        let salary_range = SalaryRange::parse(s)?;

        Ok(Some(Role {
            company: "Google".to_owned(),
            title: title.to_owned(),
            team: team.to_owned(),
            salary_range,
        }))
    }
}
