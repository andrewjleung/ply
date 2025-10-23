use anyhow::Result;
use url::Url;

use crate::{
    job::SalaryRange,
    parse::{
        ashby::Ashby, google::Google, greenhouse::Greenhouse, hiringcafe::HiringCafe, meta::Meta,
        netflix::Netflix,
    },
};

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
    Ashby,
    Google,
    Greenhouse,
    HiringCafe,
    Meta,
    Netflix,

    #[default]
    Unimplemented,
}

impl Parser {
    pub fn infer(url: &Url) -> Option<Self> {
        if url.scheme() != "https" {
            return None;
        }

        url.domain().map(|domain| match domain {
            "hiring.cafe" => Parser::HiringCafe,
            "jobs.ashbyhq.com" => Parser::Ashby,
            "job-boards.greenhouse.io" => Parser::Greenhouse,
            "www.metacareers.com" => Parser::Meta,
            "www.google.com" => Parser::Google,
            "explore.jobs.netflix.net" => Parser::Netflix,
            _ => Parser::default(),
        })
    }

    pub fn parse_role(&self, s: &str) -> Result<Option<Role>> {
        match self {
            Parser::Ashby => Ashby {}.parse(s),
            Parser::Google => Google {}.parse(s),
            Parser::Greenhouse => Greenhouse {}.parse(s),
            Parser::HiringCafe => HiringCafe {}.parse(s),
            Parser::Meta => Meta {}.parse(s),
            Parser::Netflix => Netflix {}.parse(s),
            Parser::Unimplemented => Ok(None),
        }
    }
}
