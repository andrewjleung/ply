use anyhow::{Error, Result};
use bon::Builder;
use serde::{Deserialize, Serialize};

use crate::data::id_filename;

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
    pub team: Option<String>,
    pub salary_range: Option<SalaryRange>,
}

impl Job {
    pub fn filename(&self) -> Result<String> {
        let url = self.listing_url.clone().ok_or_else(|| {
            Error::msg("cannot create unique filename for job without a listing URL")
        })?;

        let mut attrs = vec![self.company.to_owned(), self.title.to_owned()];
        if let Some(team) = &self.team {
            attrs.push(team.to_owned());
        }

        Ok(id_filename(url.as_str(), attrs))
    }
}
