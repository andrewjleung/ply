use anyhow::{Context, Result};
use bon::Builder;
use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fs::File;

use crate::{
    PlyConfig,
    document::{Document, Filename, PreDocument},
    job::Job,
};

#[derive(Builder, Serialize, Deserialize, Clone)]
pub struct Application {
    pub job: Job,

    #[builder(default = Utc::now())]
    pub applied_at: DateTime<Utc>,

    pub cycle: Option<String>,

    pub stages: Vec<Stage>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize)]
pub enum StageType {
    Applied,
    Screen,
    Technical,
    Behavioral,
    Negotiation,
    Rejected,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Stage {
    pub start_time: DateTime<Utc>,
    pub deadline: Option<DateTime<Utc>>,
    pub name: Option<String>,
    pub stage_type: StageType,
}

pub fn new(job: Job) -> Application {
    let now = Utc::now();

    Application::builder()
        .applied_at(now)
        .job(job)
        .stages(vec![Stage {
            start_time: now,
            deadline: None,
            stage_type: StageType::Applied,
            name: None,
        }])
        .build()
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
        name.to_lowercase().replace(" ", "_").replace(".", "")
    }
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

impl PreDocument for Application {
    fn pre_document(&self) -> Self {
        let mut record = self.to_owned();
        record.stages.sort_by_key(|stage| stage.start_time);
        record
    }
}
