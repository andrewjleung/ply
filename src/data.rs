use std::fs::DirBuilder;

use anyhow::{Context, Error, Result};
use camino::Utf8Path as Path;

pub fn normalize_filename_attr(name: &str) -> String {
    name.to_lowercase()
        .replace(" ", "_")
        .replace(".", "")
        .replace("(", "")
        .replace(")", "")
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
