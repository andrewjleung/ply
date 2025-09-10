use anyhow::{Error, Result};
use bon::Builder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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

// TODO: dedup
fn normalize_filename_attribute(name: &str) -> String {
    name.to_lowercase().replace(" ", "_")
}

impl Job {
    pub fn filename(&self) -> Result<String> {
        let url = self.listing_url.clone().ok_or_else(|| {
            Error::msg("cannot create unique filename for job without a listing URL")
        })?;
        let hash = Sha256::digest(url.as_str());
        let elements: Vec<String> = vec![
            hex::encode(hash).chars().take(7).collect(),
            normalize_filename_attribute(&self.company),
            normalize_filename_attribute(&self.title),
            normalize_filename_attribute(&self.team),
            String::from("md"),
        ];

        Ok(elements.join("."))
    }
}
