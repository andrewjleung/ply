use anyhow::{Context, Result};
use clap::Args;

use crate::{
    application,
    command::Run,
    config,
    document::Filename,
    scrape::{self, JobScraperKind, scrape},
};
use url::Url;

#[derive(Args)]
pub struct To {
    /// The URL of the job listing
    pub url: String,

    #[arg(value_enum)]
    /// The scraper to use, this will be inferred when the given URL scheme is 'https' and required
    /// if it is 'file'
    pub scraper: Option<JobScraperKind>,
}

impl Run for To {
    fn run(&self, config: &config::PlyConfig) -> Result<()> {
        let url = Url::parse(&self.url).context("failed to parse given URL")?;
        let mut scraped = scrape(&url, self.scraper).context("failed to scrape URL")?;
        let app = application::new(scraped.job.clone());

        // TODO: handle repeat applications to the same listing
        scrape::snapshot_content(
            &mut scraped.content,
            &config.data_dir.join("listings"),
            &scraped.job.filename()?,
        )
        .context("failed to snapshot content")?;
        app.write_new_document(config)?;

        println!("application created at {}", app.filename());

        Ok(())
    }
}
