use anyhow::{Context, Result};
use bon::Builder;
use camino::{Utf8Path as Path, Utf8PathBuf as PathBuf};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
};

use crate::{
    PlyConfig,
    document::{Document, Filename},
    scraper,
};

#[derive(Serialize, Deserialize, Clone)]
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
#[derive(Builder, Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub enum StageType {
    Application,
    Screen,
    Technical,
    Behavioral,
    Negotiation,
    Rejected,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Stage {
    start_time: DateTime<Utc>,
    deadline: Option<DateTime<Utc>>,
    name: Option<String>,
    stage_type: StageType,
}

pub fn new(company: &str, title: &str, team: &str) -> Application {
    let now = Utc::now();
    Application::builder()
        .applied_at(now)
        .company(company.to_owned())
        .title(title.to_owned())
        .team(team.to_owned())
        .stages(vec![Stage {
            start_time: now,
            deadline: None,
            stage_type: StageType::Application,
            name: None,
        }])
        .build()
}

impl Filename for Application {
    fn filename(&self) -> String {
        let elements: Vec<String> = vec![
            self.applied_at.format("%Y%m%d%H%M%S%3f").to_string(),
            Self::normalize_filename_attribute(&self.company),
            Self::normalize_filename_attribute(&self.title),
            Self::normalize_filename_attribute(&self.team),
            String::from("md"),
        ];

        elements.join(".")
    }
}

impl Application {
    pub fn write_new_document(&self, config: &PlyConfig) -> Result<File> {
        let doc: Document<Application> = Document {
            record: self.to_owned(),
            content: None,
        };

        doc.write_new(&config.data_dir)
            .context("failed to write application")
    }

    pub fn snap(&self, destination: &Path) -> Result<File> {
        unimplemented!()
    }

    fn normalize_filename_attribute(name: &str) -> String {
        name.to_lowercase().replace(" ", "_")
    }
}
