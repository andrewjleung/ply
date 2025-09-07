use std::{
    fs::{DirBuilder, File},
    io::Write,
};

use anyhow::{Context, Error, Result};
use camino::Utf8Path as Path;
use serde::Serialize;

pub trait Filename {
    fn filename(&self) -> String;
}

pub struct Document<Documentable: Serialize + Filename> {
    pub record: Documentable,
    pub content: Option<String>,
}

fn ensure_directory(dir: &Path) -> Result<()> {
    if dir.is_file() {
        return Err(Error::msg(format!(
            "failed to create directory {dir}: is a file"
        )));
    }

    DirBuilder::new()
        .recursive(true)
        .create(dir)
        .context("failed to create destination directory for document")
}

impl<Documentable: Serialize + Filename> Document<Documentable> {
    pub fn write_new(&self, dir: &Path) -> Result<File> {
        ensure_directory(dir)?;

        let filename = self.record.filename();
        let mut f = File::create_new(dir.join(filename)).context("failed to create document")?;
        let frontmatter = toml::to_string(&self.record)
            .context("failed to serialize frontmatter for document")?;

        let content = format!(
            "---\n{frontmatter}---\n\n{}",
            self.content.clone().unwrap_or("".to_owned())
        );

        f.write_all(content.as_bytes())
            .context("failed to write document")?;

        Ok(f)
    }
}
