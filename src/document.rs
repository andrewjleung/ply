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

pub struct Document<Documentable: Serialize + DeserializeOwned + Filename> {
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
    Documentable: Serialize + DeserializeOwned + Filename,
{
    let file = File::open(filename).context(format!("failed to open document at {}", filename))?;

    let mut frontmatter = String::new();
    let mut content = String::new();
    let mut in_frontmatter = false;
    for line in BufReader::new(file).lines() {
        let line = line.context(format!("failed to read line in document at {}", filename))?;
        if line == "---" {
            if in_frontmatter {
                break;
            } else {
                in_frontmatter = true;
            }
        } else {
            if in_frontmatter {
                frontmatter.push_str(&line);
                frontmatter.push_str("\n");
            } else {
                content.push_str(&line);
                content.push_str("\n");
            }
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

impl<Documentable: Serialize + DeserializeOwned + Filename> Document<Documentable> {
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

    pub fn write(&self, dir: &Path) -> Result<File> {
        ensure_directory(dir)?;

        let filename = self.record.filename();
        let mut f = File::create(dir.join(filename)).context("failed to create document")?;
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
