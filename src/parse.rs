use anyhow::Result;
use url::Url;

use crate::{
    job::SalaryRange,
    parse::{
        apple::Apple, ashby::Ashby, google::Google, greenhouse::Greenhouse, hiringcafe::HiringCafe,
        meta::Meta, mini::Mini, netflix::Netflix,
    },
};

pub mod apple;
pub mod ashby;
pub mod google;
pub mod greenhouse;
pub mod hiringcafe;
pub mod meta;
pub mod mini;
pub mod netflix;
pub mod salary;

pub trait Parse<Parsable, Parsed>
where
    Parsed: Sized,
{
    fn parse(&self, p: Parsable) -> Result<Option<Parsed>>;
}

pub trait ParseSelf<Parsable>
where
    Self: Sized,
{
    fn parse(p: Parsable) -> Result<Option<Self>>;
}

#[derive(Debug)]
pub struct Role {
    pub company: String,
    pub title: String,
    pub team: Option<String>,
    pub salary_range: Option<SalaryRange>,
}

#[allow(dead_code)]
#[derive(Default, Clone, clap::ValueEnum)]
pub enum Parser {
    Apple,
    Ashby,
    Google,
    Greenhouse,
    HiringCafe,
    Meta,
    Netflix,
    DataDog,

    #[default]
    Unimplemented,
}

impl Parser {
    pub fn infer(url: &Url) -> Option<Self> {
        if url.scheme() != "https" {
            return None;
        }

        url.domain().map(|domain| match domain {
            "careers.datadoghq.com" => Parser::DataDog,
            "explore.jobs.netflix.net" => Parser::Netflix,
            "hiring.cafe" => Parser::HiringCafe,
            "job-boards.greenhouse.io" => Parser::Greenhouse,
            "jobs.ashbyhq.com" => Parser::Ashby,
            "www.google.com" => Parser::Google,
            "www.metacareers.com" => Parser::Meta,
            "jobs.apple.com" => Parser::Apple,
            _ => Parser::default(),
        })
    }

    pub fn parse_role(&self, s: &str) -> Result<Option<Role>> {
        match self {
            Parser::Apple => Apple {}.parse(s),
            Parser::Ashby => Ashby {}.parse(s),
            Parser::Google => Google {}.parse(s),
            Parser::Greenhouse => Greenhouse {}.parse(s),
            Parser::HiringCafe => HiringCafe {}.parse(s),
            Parser::Meta => Meta {}.parse(s),
            Parser::Netflix => Netflix {}.parse(s),
            Parser::DataDog => Mini {
                company: "DataDog".to_owned(),
                title_and_team_selector: "head > title".to_owned(),
                title_and_team_regex: None,
                salary_range_selector: ".pay-range".to_owned(),
                salary_range_regex: None,
            }
            .parse(s),
            Parser::Unimplemented => Ok(None),
        }
    }
}
