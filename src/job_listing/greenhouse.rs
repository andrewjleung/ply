use anyhow::{Context, Result, anyhow};
use regex::Regex;
use scraper::{Html, Selector};
use url::Url;

use crate::{
    job::{Job, SalaryRange},
    parse::Parse,
};

#[derive(Default, Debug)]
pub struct Greenhouse {
    pub url: Option<Url>,
}

impl Greenhouse {
    fn parse_company_title_and_team(
        &self,
        document: &Html,
    ) -> Result<(String, String, Option<String>)> {
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

impl Parse<Greenhouse> for Job {
    fn parse_with_config(s: &str, config: &Greenhouse) -> Result<Option<Self>> {
        let document = Html::parse_document(s);
        let (company, title, team) = config
            .parse_company_title_and_team(&document)
            .context("failed to parse company, title, and team")?;
        let salary_range = SalaryRange::parse(&s)?;

        Ok(Some(Job {
            listing_url: config.url.to_owned(),
            company: company.to_owned(),
            title: title.to_owned(),
            team: team.to_owned(),
            salary_range,
        }))
    }
}
