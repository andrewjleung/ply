use anyhow::{Context, Error, Result};
use camino::Utf8Path as Path;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    fs::{DirBuilder, File},
    io::{BufRead, BufReader, BufWriter, Write},
};

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

pub fn read<Documentable>(filename: &Path) -> Result<Document<Documentable>>
where
    Documentable: Serialize + DeserializeOwned + Filename + Clone + PreDocument,
{
    let file = File::open(filename).context(format!("failed to open document at {}", filename))?;

    let mut frontmatter = String::new();
    let mut content = String::new();
    let mut in_frontmatter = false;
    for line in BufReader::new(file).lines() {
        let line = line.context(format!("failed to read line in document at {}", filename))?;
        if line == "---" {
            in_frontmatter = !in_frontmatter;
        } else if in_frontmatter {
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
    pub fn write_new(&self, dir: &Path) -> Result<File> {
        ensure_directory(dir)?;

        let filename = self.record.filename();
        let mut f = File::create_new(dir.join(filename))
            .context(format!("failed to create document at {}", dir.as_str()))?;

        let record = self.record.pre_document();
        let frontmatter =
            toml::to_string(&record).context("failed to serialize frontmatter for document")?;

        let content = format!(
            "---\n{frontmatter}---\n\n{}",
            self.content.clone().unwrap_or("".to_owned())
        );

        f.write_all(content.as_bytes())
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
            "---\n{frontmatter}---\n\n{}",
            self.content.clone().unwrap_or("".to_owned())
        );

        f.write_all(content.as_bytes())
            .context("failed to write document")?;

        Ok(f)
    }
}
