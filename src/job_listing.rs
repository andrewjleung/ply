use url::Url;

use crate::{
    job::Job,
    job_listing::{ashby::Ashby, greenhouse::Greenhouse, meta::Meta},
    parse::Parse,
};

pub mod ashby;
pub mod greenhouse;
pub mod meta;

#[derive(Default, Debug)]
pub enum JobListing {
    Ashby(Ashby),
    Google,
    Greenhouse(Greenhouse),
    HiringCafe,
    Meta(Meta),
    Netflix,

    #[default]
    Unimplemented,
}

impl JobListing {
    pub fn infer(url: &Url) -> Option<Self> {
        if url.scheme() != "https" {
            return None;
        }

        url.domain().map(|domain| match domain {
            // "hiring.cafe" => JobListing::HiringCafe,
            "jobs.ashbyhq.com" => JobListing::Ashby(Ashby {
                url: Some(url.to_owned()),
            }),
            "job-boards.greenhouse.io" => JobListing::Greenhouse(Greenhouse {
                url: Some(url.to_owned()),
            }),
            "www.metacareers.com" => JobListing::Meta(Meta {
                url: Some(url.to_owned()),
            }),
            // "www.google.com" => JobListing::Google,
            // "explore.jobs.netflix.net" => JobListing::Netflix,
            _ => JobListing::default(),
        })
    }
}

impl Parse<JobListing> for Job {
    fn parse_with_config(s: &str, config: &JobListing) -> anyhow::Result<Option<Self>> {
        match config {
            JobListing::Meta(m) => Job::parse_with_config(s, m),
            JobListing::Greenhouse(g) => Job::parse_with_config(s, g),
            jb => unimplemented!("job parsing is unimplemented for {:?}", jb),
        }
    }
}
