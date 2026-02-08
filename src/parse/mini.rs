use std::rc::Rc;

use anyhow::{Context, Result, anyhow};
use regex::Regex;
use scraper::{Html, Selector};

use crate::{
    job::SalaryRange,
    parse::{Parse, ParseSelf, Role},
};

enum ParseStrategy {
    Json {
        data: Rc<serde_json::Value>,
        path: Vec<String>,
        regex: Option<Rc<Regex>>,
        regex_key: Option<Rc<str>>,
    },
    Html {
        data: Rc<scraper::Html>,
        selector: Rc<scraper::Selector>,
        regex: Option<Rc<Regex>>,
        regex_key: Option<Rc<str>>,
    },
}

impl ParseStrategy {
    pub fn parse(&self) -> Result<String> {
        match self {
            Self::Json {
                data,
                path,
                regex,
                regex_key,
            } => {
                let root: &serde_json::Value = data.as_ref();
                let value = path
                    .iter()
                    .try_fold(root, |acc, key| {
                        acc.get(key)
                            .ok_or_else(|| anyhow!("key '{}' not found", key))
                    })
                    .context(format!("failed to parse path '{}'", path.join(".")));

                let value = value
                    .and_then(|v| v.as_str().ok_or(anyhow!("failed to parse value as string")))?;

                if let Some(re) = regex {
                    re.captures(value)
                        .ok_or(anyhow!("no regex matches found in keyed data"))
                        .and_then(|captures| match regex_key {
                            Some(key) => captures
                                .name(key)
                                .map(|m| m.as_str().trim().to_string())
                                .ok_or(anyhow!(
                                    "no capture group with key '{}' found in keyed data",
                                    key
                                )),
                            None => captures
                                .iter()
                                .next()
                                .flatten()
                                .ok_or(anyhow!("no capture groups in regex match for keyed data"))
                                .map(|capture| capture.as_str().into()),
                        })
                } else {
                    Ok(value.into())
                }
            }
            Self::Html {
                data,
                selector,
                regex,
                regex_key,
            } => {
                let selection = data
                    .select(selector)
                    .next()
                    .context("no matches for selector in document")?
                    .text()
                    .collect::<Vec<_>>()
                    .join("");

                if let Some(re) = regex {
                    re.captures(&selection)
                        .ok_or(anyhow!("no regex matches found in selected data"))
                        .and_then(|captures| match regex_key {
                            Some(key) => captures
                                .name(key)
                                .map(|m| m.as_str().trim().to_string())
                                .ok_or(anyhow!(
                                    "no capture group with key '{}' found in keyed data",
                                    key
                                )),
                            None => captures
                                .iter()
                                .next()
                                .flatten()
                                .ok_or(anyhow!("no capture groups in regex match for keyed data"))
                                .map(|capture| capture.as_str().into()),
                        })
                } else {
                    Ok(selection)
                }
            }
        }
    }
}

pub struct Mini {
    pub company: String,
    pub title_and_team_selector: String,
    pub title_and_team_regex: Option<Rc<Regex>>,
    pub salary_range_selector: String,
    pub salary_range_regex: Option<Rc<Regex>>,
}

impl Mini {
    fn parse_title_and_team(&self, document: Rc<Html>) -> Result<(String, Option<String>)> {
        let document_title_selector = Rc::new(
            Selector::parse(&self.title_and_team_selector)
                .expect("failed to compile title selector"),
        );

        let title_re = self.title_and_team_regex.as_ref().map(Rc::clone).unwrap_or(
            Regex::new(
                r"^(?P<title>[A-Za-z\s/&()]+?(?:\s+[IVX]+)?)\s*(?:[-–—,]\s*(?:[^|]+))?(?:\s*\|\s*.*)?$",
            )
            .unwrap()
            .into(),
        );

        let team_re = self.title_and_team_regex.as_ref().map(Rc::clone).unwrap_or(
            Regex::new(
                r"^(?:[A-Za-z\s/&()]+?(?:\s+[IVX]+)?)\s*(?:[-–—,]\s*(?P<team>[^|]+))?(?:\s*\|\s*.*)?$",
            )
            .unwrap()
            .into(),
        );

        let title = ParseStrategy::Html {
            data: Rc::clone(&document),
            selector: Rc::clone(&document_title_selector),
            regex: Some(title_re),
            regex_key: Some(Rc::from("title")),
        }
        .parse()
        .context("failed to parse team")?;

        let team = ParseStrategy::Html {
            data: Rc::clone(&document),
            selector: Rc::clone(&document_title_selector),
            regex: Some(team_re),
            regex_key: Some(Rc::from("team")),
        }
        .parse()
        .ok();

        Ok((title, team))
    }

    fn parse_salary_range(&self, document: Rc<Html>) -> Result<Option<SalaryRange>> {
        let salary_range_selector = Rc::new(
            Selector::parse(&self.salary_range_selector)
                .expect("failed to compile salary range selector"),
        );

        let salary_range = ParseStrategy::Html {
            data: document,
            selector: salary_range_selector,
            regex: self.salary_range_regex.as_ref().map(Rc::clone),
            regex_key: None,
        }
        .parse()
        .context("failed to parse salary range")?;

        SalaryRange::parse(&salary_range)
    }
}

impl Parse<&str, Role> for Mini {
    fn parse(&self, s: &str) -> Result<Option<Role>> {
        let document = Rc::new(Html::parse_document(s));
        let (title, team) = self
            .parse_title_and_team(Rc::clone(&document))
            .context("failed to parse title and team")?;

        let salary_range = self.parse_salary_range(document)?;

        Ok(Some(Role {
            company: self.company.to_owned(),
            title: title.to_owned(),
            team: team.to_owned(),
            salary_range,
        }))
    }
}
