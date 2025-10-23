use anyhow::{Context, Result, anyhow};
use regex::Regex;
use scraper::{Html, Selector};

use crate::{job::SalaryRange, parse::Parse, parse::Role};

pub struct Greenhouse {}

impl Greenhouse {
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

        let pipe_delim_re =
            Regex::new(r"^(?P<title>.*?) (?:- (?P<team>.*?))(?: \([^)]*\))?\s*\| (?P<company>.*)$")
                .unwrap();

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

        if let Some(caps) = pipe_delim_re.captures(&document_title) {
            let title = caps.name("title").unwrap().as_str().trim().to_string();
            let team = caps.name("team").map(|m| m.as_str().trim().to_string());
            let company = caps.name("company").unwrap().as_str().trim().to_string();
            return Ok((company, title, team));
        }

        Err(anyhow!("failed to match title {document_title}"))
    }
}

impl Parse<&str, Role> for Greenhouse {
    fn parse(s: &str) -> Result<Option<Role>> {
        let document = Html::parse_document(s);
        let (company, title, team) = Self::parse_company_title_and_team(&document)
            .context("failed to parse company, title, and team")?;
        let salary_range = SalaryRange::parse(s)?;

        Ok(Some(Role {
            company: company.to_owned(),
            title: title.to_owned(),
            team: team.to_owned(),
            salary_range,
        }))
    }
}
