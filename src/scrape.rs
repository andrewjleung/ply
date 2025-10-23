use crate::fetch::{Fetch, Source};
use crate::parse::Parser;
use anyhow::{Context, Error, Result};
use camino::Utf8Path as Path;
use camino::Utf8PathBuf as PathBuf;
use std::{
    fs::{DirBuilder, File},
    io::Write,
};
use url::Url;

use crate::job::Job;

pub struct ScrapedContent {
    pub job: Job,
    pub content: String,
}

impl ScrapedContent {
    pub fn from_url(url: &Url) -> Result<Option<Self>> {
        let listing = if let Some(parser) = Parser::infer(url) {
            let content = Source::try_from(url)?.fetch()?;
            let role = parser.parse_role(&content)?;

            role.map(|role| ScrapedContent {
                job: Job {
                    listing_url: Some(url.to_owned()),
                    company: role.company,
                    title: role.title,
                    team: role.team,
                    salary_range: role.salary_range,
                },
                content,
            })
        } else {
            None
        };

        Ok(listing)
    }

    pub fn snapshot(&self, content_dir: &Path, filename: &str) -> Result<PathBuf> {
        if content_dir.is_file() {
            return Err(Error::msg(format!(
                "content directory {} is a file, not a directory",
                content_dir
            )));
        }

        DirBuilder::new()
            .recursive(true)
            .create(content_dir)
            .context(format!(
                "failed to create content directory {} for scraped content",
                content_dir
            ))?;

        let filepath = content_dir.join(filename);

        if filepath.try_exists().context(format!(
            "failed to determine if snapshotted content already exists at {}",
            filepath
        ))? {
            return Ok(filepath);
        };

        let mut f = File::create_new(&filepath).context(format!(
            "failed to create file {} for scraped content",
            filepath
        ))?;

        let markdown_content = htmd::HtmlToMarkdown::builder()
            .skip_tags(vec!["style"])
            .build()
            .convert(&self.content)
            .context("failed to convert scraped HTML to markdown")?;

        f.write(markdown_content.as_bytes())
            .context("failed to write scraped markdown content")?;

        Ok(filepath)
    }
}
