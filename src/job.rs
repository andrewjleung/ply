use anyhow::{Context, Result};
use bon::Builder;
use camino::{Utf8Path as Path, Utf8PathBuf as PathBuf};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct SalaryRange {
    pub lower: u32,
    pub range: Option<u32>,
}

impl SalaryRange {
    pub fn upper(&self) -> Option<u32> {
        self.range.map(|r| self.lower + r)
    }
}

#[derive(Builder, Serialize, Deserialize, Clone)]
pub struct Job {
    pub listing_url: Option<url::Url>,

    pub company: String,

    pub title: String,

    pub team: String,

    pub salary_range: Option<SalaryRange>,
}
