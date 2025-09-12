use std::ops::Not;

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

    /// The scraper to use, this will be inferred when the given URL scheme is 'https' and required
    /// if it is 'file'
    #[arg(value_enum, long, short)]
    pub scraper: Option<JobScraperKind>,

    // The job application cycle for this application
    #[arg(long, short)]
    pub cycle: Option<String>,
}

impl Run for To {
    fn run(&self, config: &config::PlyConfig) -> Result<()> {
        let url = Url::parse(&self.url).context("failed to parse given URL")?;
        let mut scraped = scrape(&url, self.scraper).context("failed to scrape URL")?;
        let app = application::new(
            scraped.job.to_owned(),
            self.cycle
                .to_owned()
                .and_then(|c| c.is_empty().not().then_some(c))
                .to_owned()
                .or(config.default_cycle.to_owned()),
        );

        // TODO: handle repeat applications to the same listing
        if let Ok(filename) = scraped.job.filename() {
            scrape::snapshot_content(
                &mut scraped.content,
                &config.data_dir.join("listings"),
                &filename,
            )
            .context("failed to snapshot content")?;
        }

        app.write_new_document(config)?;

        println!("application created at {}", app.filename());

        Ok(())
    }
}
