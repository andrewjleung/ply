use anyhow::{Context, Result, anyhow};
use regex::Regex;
use scraper::{Html, Selector};
use url::Url;

use crate::{
    job::{Job, SalaryRange},
    parse::Parse,
};

#[derive(Default, Debug)]
pub struct Meta {
    pub url: Option<Url>,
}

impl Meta {
    fn parse_title_and_team(&self, document: &Html) -> Result<(String, Option<String>)> {
        let document_title_selector =
            Selector::parse("#pageTitle").expect("failed to compile title selector");

        let document_title = &document
            .select(&document_title_selector)
            .next()
            .context("failed to select document title from document")?
            .text()
            .collect::<Vec<_>>()
            .join("");

        let document_title = html_escape::decode_html_entities(document_title);
        let title_re = Regex::new(r"^(?P<title>[^,]+),\s*(?P<team>[^|]+)\s*\|").unwrap();

        if let Some(caps) = title_re.captures(&document_title) {
            let title = caps.name("title").unwrap().as_str().trim().to_string();
            let team = caps.name("team").map(|m| m.as_str().trim().to_string());
            return Ok((title, team));
        }

        Err(anyhow!("failed to match title {document_title}"))
    }

    fn parse_salary_range(&self, s: &str) -> Result<Option<SalaryRange>> {
        Regex::new(r">\$.*to.*\$.*bonus \+ equity \+ benefits")
            .unwrap()
            .captures(s)
            .and_then(|captures| captures.iter().next().flatten())
            .and_then(|line| SalaryRange::parse(line.as_str()).transpose())
            .transpose()
    }
}

impl Parse<Meta> for Job {
    fn parse_with_config(s: &str, config: &Meta) -> Result<Option<Self>> {
        let document = Html::parse_document(s);
        let (title, team) = config
            .parse_title_and_team(&document)
            .context("failed to parse title and team")?;

        let salary_range = config.parse_salary_range(s)?;

        Ok(Some(Job {
            listing_url: config.url.to_owned(),
            company: "Meta".to_string(),
            title: title.to_owned(),
            team: team.to_owned(),
            salary_range,
        }))
    }
}
