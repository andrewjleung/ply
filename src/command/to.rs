use std::ops::Not;

use anyhow::{Context, Result};
use clap::Args;

use crate::{
    application,
    command::Run,
    config,
    document::Filename,
    job,
    scrape::{self, JobScraperKind, scrape},
};
use url::Url;

#[derive(Args)]
pub struct To {
    /// The URL of the job listing
    pub url: Option<String>,

    /// The scraper to use, this will be inferred when the given URL scheme is 'https' and required
    /// if it is 'file'
    #[arg(value_enum, long, short, requires("url"))]
    pub scraper: Option<JobScraperKind>,

    /// The company for a new application, required if no listing URL is given
    #[arg(long, conflicts_with("url"), required_unless_present("url"))]
    pub company: Option<String>,

    /// The job title for a new application, required if no listing URL is given
    #[arg(long, conflicts_with("url"), required_unless_present("url"))]
    pub title: Option<String>,

    /// The job team for a new application
    #[arg(long, conflicts_with("url"))]
    pub team: Option<String>,

    // The job application cycle for this application
    #[arg(long, short)]
    pub cycle: Option<String>,

    // Print the application to STDOUT instead of writing it
    #[arg(long, short)]
    pub print: bool,
}

impl Run for To {
    fn run(&self, config: &config::PlyConfig) -> Result<()> {
        match &self.url {
            Some(url) => {
                let url = Url::parse(url).context("failed to parse given URL")?;
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

                if self.print {
                    println!("{}", app.new_document().new_content()?);
                } else {
                    app.write_new_document(config)?;
                    println!("application created at {}", app.filename());
                }

                Ok(())
            }
            None => {
                let job = job::Job {
                    company: self.company.to_owned().unwrap(),
                    title: self.title.to_owned().unwrap(),
                    team: self.team.to_owned(),
                    listing_url: None,
                    salary_range: None,
                };

                let app = application::new(job, self.cycle.clone());

                if self.print {
                    println!("{}", app.new_document().new_content()?);
                } else {
                    app.write_new_document(config)?;
                    println!("application created at {}", app.filename());
                }

                println!("application created at {}", app.filename());

                Ok(())
            }
        }
    }
}
