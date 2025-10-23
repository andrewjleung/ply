use anyhow::Result;
use url::Url;

use crate::{
    fetch::{Fetch, Source},
    job::{Job, SalaryRange},
    parse::{
        Parse, ashby::Ashby, google::Google, greenhouse::Greenhouse, hiringcafe::HiringCafe,
        meta::Meta,
    },
    scrape::ScrapedContent,
};

#[derive(Debug)]
pub struct Role {
    pub company: String,
    pub title: String,
    pub team: Option<String>,
    pub salary_range: Option<SalaryRange>,
}

#[allow(dead_code)]
#[derive(Default)]
pub enum JobSource {
    Ashby(Url),
    Google(Url),
    Greenhouse(Url),
    HiringCafe(Url),
    Meta(Url),
    Netflix(Url),

    #[default]
    Unimplemented,
}

impl JobSource {
    pub fn infer(url: &Url) -> Option<Self> {
        if url.scheme() != "https" {
            return None;
        }

        url.domain().map(|domain| match domain {
            "hiring.cafe" => JobSource::HiringCafe(url.to_owned()),
            "jobs.ashbyhq.com" => JobSource::Ashby(url.to_owned()),
            "job-boards.greenhouse.io" => JobSource::Greenhouse(url.to_owned()),
            "www.metacareers.com" => JobSource::Meta(url.to_owned()),
            "www.google.com" => JobSource::Google(url.to_owned()),
            // "explore.jobs.netflix.net" => JobListing::Netflix,
            _ => JobSource::default(),
        })
    }

    pub fn parse_role(&self, s: &str) -> Result<Option<Role>> {
        match self {
            JobSource::Ashby(_) => Ashby::parse(s),
            JobSource::Google(_) => Google::parse(s),
            JobSource::Greenhouse(_) => Greenhouse::parse(s),
            JobSource::HiringCafe(_) => HiringCafe::parse(s),
            JobSource::Meta(_) => Meta::parse(s),
            _ => Ok(None),
        }
    }
}

impl ScrapedContent {
    pub fn from_url(url: &Url) -> Result<Option<Self>> {
        let job_source = JobSource::infer(url);

        let listing = if let Some(job_source) = job_source {
            let content = Source::try_from(url)?.fetch()?;
            let role = job_source.parse_role(&content)?;

            role.map(|role| ScrapedContent {
                job: Job {
                    listing_url: Some(url.to_owned()),
                    company: role.company,
                    title: role.title,
                    team: role.team,
                    salary_range: role.salary_range,
                },
                content,
            })
        } else {
            None
        };

        Ok(listing)
    }
}
