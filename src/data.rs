use std::{fmt::Display, fs::DirBuilder};

use anyhow::{Context, Error, Result};
use camino::Utf8Path as Path;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

const TIMESTAMP_FORMAT: &str = "%Y%m%d%H%M%S%3f";
const HASH_ID_LENGTH: usize = 7;

pub fn normalize_filename_attr(name: &str) -> String {
    name.to_lowercase()
        .replace(" - ", "_")
        .replace(" ", "_")
        .replace("(", "")
        .replace(")", "")
        .replace("-", "_")
        .replace(".", "")
        .replace("/", "_")
}

pub fn ensure_directory(dir: &Path) -> Result<()> {
    if dir.is_file() {
        return Err(Error::msg(format!(
            "failed to create directory {dir}: is a file"
        )));
    }

    DirBuilder::new()
        .recursive(true)
        .create(dir)
        .context(format!("failed to build directory {dir}"))
}

pub fn timestamp_filename(timestamp: &DateTime<Utc>, attrs: Vec<impl Display>) -> String {
    format!(
        "{}.{}.md",
        timestamp.format(TIMESTAMP_FORMAT),
        attrs
            .iter()
            .map(|attr| normalize_filename_attr(&attr.to_string()))
            .collect::<Vec<String>>()
            .join(".")
    )
}

pub fn id_filename(id: &str, attrs: Vec<impl Display>) -> String {
    let hash = Sha256::digest(id);

    format!(
        "{}.{}.md",
        hex::encode(hash)
            .chars()
            .take(HASH_ID_LENGTH)
            .collect::<String>(),
        attrs
            .iter()
            .map(|attr| normalize_filename_attr(&attr.to_string()))
            .collect::<Vec<String>>()
            .join(".")
    )
}
