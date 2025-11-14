use anyhow::{Context, Result};
use camino::Utf8Path as Path;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
};

use crate::data::ensure_directory;

pub trait Filename {
    fn filename(&self) -> String;
}

/// This trait can be implemented to allow for a documentable to apply some
/// transformation to itself before being documented, for instance sorting
/// attributes.
pub trait PreDocument
where
    Self: Sized + Clone,
{
    fn pre_document(&self) -> Self {
        self.to_owned()
    }
}

pub struct Document<Documentable: Serialize + DeserializeOwned + Filename + Clone + PreDocument> {
    pub record: Documentable,
    pub content: Option<String>,
}

pub fn read<Documentable>(filename: &Path) -> Result<Document<Documentable>>
where
    Documentable: Serialize + DeserializeOwned + Filename + Clone + PreDocument,
{
    let file = File::open(filename).context(format!("failed to open document at {}", filename))?;

    let mut frontmatter = String::new();
    let mut content = String::new();
    let mut state = 0;

    for line in BufReader::new(file).lines() {
        let line = line.context(format!("failed to read line in document at {}", filename))?;

        if line.trim() == "---" {
            state += 1;
        } else if state == 1 {
            frontmatter.push_str(&line);
            frontmatter.push('\n');
        } else {
            content.push_str(&line);
            content.push('\n');
        };
    }

    let record: Documentable = toml::from_str(&frontmatter).context(format!(
        "failed to deserialize document at {} into TOML",
        filename
    ))?;

    Ok(Document::<Documentable> {
        record,
        content: Some(content),
    })
}

impl<Documentable: Serialize + DeserializeOwned + Filename + Clone + PreDocument>
    Document<Documentable>
{
    pub fn new_content(&self) -> Result<String> {
        let record = self.record.pre_document();
        let frontmatter =
            toml::to_string(&record).context("failed to serialize frontmatter for document")?;

        Ok(format!(
            "---\n{frontmatter}---\n{}",
            self.content.clone().unwrap_or("".to_owned())
        ))
    }

    pub fn write_new(&self, dir: &Path) -> Result<File> {
        ensure_directory(dir)?;

        let filename = self.record.filename();
        let mut f = File::create_new(dir.join(filename))
            .context(format!("failed to create document at {}", dir.as_str()))?;

        f.write_all(self.new_content()?.as_bytes())
            .context("failed to write document")?;

        Ok(f)
    }

    pub fn write(&self, dir: &Path) -> Result<File> {
        ensure_directory(dir)?;

        let filename = self.record.filename();
        let mut f = File::create(dir.join(filename)).context("failed to create document")?;
        let record = self.record.pre_document();
        let frontmatter =
            toml::to_string(&record).context("failed to serialize frontmatter for document")?;

        let content = format!(
            "---\n{frontmatter}---\n{}",
            self.content.clone().unwrap_or("".to_owned())
        );

        f.write_all(content.as_bytes())
            .context("failed to write document")?;

        Ok(f)
    }
}
