use anyhow::{Context, Result};
use bon::Builder;
use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fs::File;

use crate::{
    PlyConfig,
    data::timestamp_filename,
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize, Debug)]
pub enum StageType {
    Applied,
    Screen,
    Technical,
    Behavioral,
    Negotiation,
    Rejected,
    Accepted,
}

impl StageType {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Rejected | Self::Accepted)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Stage {
    pub start_time: DateTime<Utc>,
    pub deadline: Option<DateTime<Utc>>,
    pub name: Option<String>,
    pub stage_type: StageType,
}

pub fn new(job: Job, cycle: Option<String>) -> Application {
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
        .maybe_cycle(cycle)
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

    pub fn current_stage(&self) -> Option<Stage> {
        let mut stages = self.stages.clone();
        stages.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        stages.last().cloned()
    }

    pub fn is_active(&self) -> bool {
        match self.current_stage() {
            Some(stage) => !stage.stage_type.is_terminal(),
            None => true,
        }
    }
}

impl Filename for Application {
    fn filename(&self) -> String {
        let mut attrs = vec![self.job.company.to_owned(), self.job.title.to_owned()];
        if let Some(team) = &self.job.team {
            attrs.push(team.to_owned());
        }

        timestamp_filename(&self.applied_at, attrs)
    }
}

impl PreDocument for Application {
    fn pre_document(&self) -> Self {
        let mut record = self.to_owned();
        record.stages.sort_by_key(|stage| stage.start_time);
        record
    }
}
