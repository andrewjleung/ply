use std::fs::File;

use anyhow::{Context, Result};
use bon::Builder;
use camino::{Utf8Path as Path, Utf8PathBuf as PathBuf};
use chrono::{DateTime, Utc};
use url::Url;

use crate::scraper;

pub struct SalaryRange {
    lower: u32,
    range: u32,
}

impl SalaryRange {
    pub fn upper(&self) -> u32 {
        self.lower + self.range
    }
}

/// For now, an application will be uniquely identified by:
/// (cycle, company, title, team, applied_at, listing_url)
#[derive(Builder)]
pub struct Application {
    pub listing_url: Option<url::Url>,

    #[builder(default = Utc::now())]
    pub applied_at: DateTime<Utc>,

    pub cycle: Option<String>,

    pub company: String,

    pub title: String,

    pub team: String,

    pub stages: Vec<Stage>,

    pub salary_range: Option<SalaryRange>,
}

pub enum StageType {
    Application,
    Screen,
    Technical,
    Behavioral,
    Negotiation,
    Rejected,
}

pub struct Stage {
    start_time: DateTime<Utc>,
    deadline: Option<DateTime<Utc>>,
    name: Option<String>,
    stage_type: StageType,
}

impl Application {
    pub fn document(&self) -> Result<File> {
        unimplemented!()
    }

    pub fn snap(&self, destination: &Path) -> Result<File> {
        unimplemented!()
    }

    fn normalize(name: &str) -> String {
        name.to_lowercase().replace(" ", "")
    }

    pub fn file_name(&self) -> String {
        let elements: Vec<String> = vec![
            self.applied_at.format("%Y%m%d.%k%M%S%L").to_string(),
            Self::normalize(&self.company),
            Self::normalize(&self.title),
            Self::normalize(&self.team),
            String::from("md"),
        ];

        elements.join(".")
    }
}
