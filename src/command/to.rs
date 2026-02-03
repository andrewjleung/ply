use anyhow::{Context, Result, anyhow};
use clap::Args;
use std::ops::Not;

use crate::{
    application, command::Run, config, document::Filename, job, parse::Parser,
    scrape::ScrapedContent,
};
use url::Url;

#[derive(Args)]
pub struct To {
    /// The URL of the job listing
    pub url: Option<String>,

    /// The parser to use, this will be inferred when the given URL scheme is 'https' and required
    /// if it is 'file'
    #[arg(value_enum, long, short, requires("url"))]
    pub parser: Option<Parser>,

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
    #[arg(long)]
    pub print: bool,
}

impl Run for To {
    fn run(&self, config: &config::PlyConfig) -> Result<()> {
        let cycle = self
            .cycle
            .to_owned()
            .and_then(|c| c.is_empty().not().then_some(c))
            .or(config.default_cycle.to_owned());

        let application = match &self.url {
            Some(url) => {
                let url = Url::parse(url).context("failed to parse given URL")?;

                let scraped = ScrapedContent::from_url(&url)
                    .and_then(|content| content.ok_or(anyhow!("no result from scraping URL")))
                    .context("failed to scrape URL")?;

                let app = application::new(scraped.job.to_owned(), cycle);

                // TODO: handle repeat applications to the same listing
                if let Ok(filename) = scraped.job.filename() {
                    scraped
                        .snapshot(&config.data_dir.join("listings"), &filename)
                        .context("failed to snapshot content")?;
                }

                app
            }
            None => {
                let job = job::Job {
                    company: self.company.to_owned().unwrap(),
                    title: self.title.to_owned().unwrap(),
                    team: self.team.to_owned(),
                    listing_url: None,
                    salary_range: None,
                };

                application::new(job, cycle)
            }
        };

        if self.print {
            println!("{}", application.new_document().new_content()?);
        } else {
            application.write_new_document(config)?;
            println!(
                "application for '{}' created at {}",
                application.pretty_print(),
                application.filename()
            );
        }

        Ok(())
    }
}
