use std::fs::File;

use anyhow::{Context, Result};
use bon::Builder;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    PlyConfig,
    document::{Document, Filename},
    job::Job,
};

/// For now, an application will be uniquely identified by:
/// (cycle, company, title, team, applied_at, listing_url)
#[derive(Builder, Serialize, Deserialize, Clone)]
pub struct Application {
    pub job: Job,

    #[builder(default = Utc::now())]
    pub applied_at: DateTime<Utc>,

    pub cycle: Option<String>,

    pub stages: Vec<Stage>,
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

pub fn new(job: Job) -> Application {
    let now = Utc::now();

    Application::builder()
        .applied_at(now)
        .job(job)
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
            Self::normalize_filename_attribute(&self.job.company),
            Self::normalize_filename_attribute(&self.job.title),
            Self::normalize_filename_attribute(&self.job.team),
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

    fn normalize_filename_attribute(name: &str) -> String {
        name.to_lowercase().replace(" ", "_")
    }
}
