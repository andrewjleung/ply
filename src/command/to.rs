use anyhow::{Context, Error, Result};
use clap::Args;

use crate::{
    application,
    command::Run,
    config,
    document::Filename,
    scrape::{self, JobScraper},
};
use url::Url;

#[derive(Args)]
pub struct To {
    /// The URL of the job listing
    pub url: Option<String>,

    /// The company name of the job, this will be inferred from the URL if not present
    pub company: Option<String>,
}

impl Run for To {
    fn run(&self, config: &config::PlyConfig) -> Result<()> {
        // TODO: support prompts when no url/company supplied
        let url = self
            .url
            .clone()
            .ok_or_else(|| Error::msg("support for interactive prompts is unimplemented"))?;

        let url = Url::parse(&url).context("failed to parse given URL")?;
        let mut scrape = scrape::hiring_cafe::new(&url)
            .context("failed to create hiring cafe scraper")?
            .scrape(&url)
            .context("failed to scrape content")?;
        let app = application::new(scrape.job.clone());

        // TODO: handle repeat applications to the same listing
        scrape::snapshot_content(
            &mut scrape.content,
            &config.data_dir.join("listings"),
            &scrape.job.filename()?,
        )
        .context("failed to snapshot content")?;
        app.write_new_document(config)?;

        println!("application created at {}", app.filename());

        Ok(())
    }
}
