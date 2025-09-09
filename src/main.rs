use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};

use crate::{application::StageType, config::PlyConfig};

/*
* - store each application in toml/markdown
*   - toml for structured data that needs to be parsed
*   - markdown for notes
* - configurable threshold for when to consider an application as ghosted
*
* - ply to [URL] [-e] -> log a job application
*   - store a snapshot of the page
*   - store some reference to the resume used to apply
*   - (e)ditor flag: open editor to log notes and responses to questions
*
* - ply yes [COMPANY] -> log next steps for a job application
*   - open a fuzzy finder to choose the exact listing
*   - prompt to choose next round type
*
* - ply no [COMPANY]
*
* - ply edit [COMPANY] [] -> edit a job application / notes
*   - open a fuzzy finder to choose the exact listing
*   - open editor with the TOML config / notes files
*
* - ply stats [CYCLE] -> give stats on the current or given cycle
*/

mod application;
mod config;
mod document;
mod job;
mod scrape;

#[derive(Parser)]
#[command(version, about)]
struct Ply {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    To(ToArgs),
    Yes(YesArgs),
    No(NoArgs),
    Edit(EditArgs),
}

#[derive(Args)]
struct ToArgs {
    /// The URL of the job listing
    url: Option<String>,

    /// The company name of the job, this will be inferred from the URL if not present
    company: Option<String>,

    /// Whether the URL refers to a local file
    #[arg(short, long)]
    local: bool,
}

#[derive(Args)]
struct YesArgs {
    /// The path to the application document
    path: String,

    /// The next stage
    #[arg(value_enum)]
    next_stage: StageType,

    /// The next stage deadline
    deadline: Option<String>,
}

#[derive(Args)]
struct NoArgs {
    /// The path to the application document
    path: String,
}

#[derive(Args)]
struct EditArgs {
    /// The company name of the job
    company: Option<String>,
}

mod ply {
    use crate::{
        EditArgs, NoArgs, ToArgs, YesArgs,
        application::{self, Application, Stage, StageType},
        config::PlyConfig,
        document::{self},
        scrape::{
            hiring_cafe::{HiringCafeScraper, TestHiringCafeScraper},
            scrape,
        },
    };
    use anyhow::{Context, Error, Result};
    use camino::Utf8Path as Path;
    use chrono::Utc;
    use url::Url;

    pub fn to(config: &PlyConfig, args: &ToArgs) -> Result<()> {
        // TODO: support prompts when no url/company supplied
        let url = args
            .url
            .clone()
            .ok_or_else(|| Error::msg("support for interactive prompts is unimplemented"))?;

        let url = Url::parse(&url).context("failed to parse given URL")?;

        if args.local {
            let scraper = TestHiringCafeScraper {};
            let job = scrape(&scraper, &url, Some(Path::new("data/jobs")))
                .context("failed to scrape given URL")?;

            application::new(job).write_new_document(config)?;
        } else {
            let scraper = HiringCafeScraper {
                listing_url: Some(url.clone()),
            };

            let job = scrape(&scraper, &url, Some(Path::new("data/jobs")))
                .context("failed to scrape given URL")?;

            application::new(job).write_new_document(config)?;
        }

        Ok(())
    }

    pub fn yes(config: &PlyConfig, args: &YesArgs) -> Result<()> {
        let mut document = document::read::<Application>(Path::new(&args.path))?;
        let now = Utc::now();
        let deadline = match &args.deadline {
            Some(deadline) => Some(
                tu::parse_date_args(
                    &deadline
                        .split(" ")
                        .map(|s| s.to_owned())
                        .collect::<Vec<_>>(),
                    now,
                )
                .map_err(anyhow::Error::new)
                .context("failed to parse deadline")?,
            ),
            None => None,
        };

        document.record.stages.push(Stage {
            start_time: now,
            deadline,
            name: None,
            stage_type: args.next_stage,
        });

        document
            .write(&config.data_dir)
            .context("failed to write new stage to document")?;

        Ok(())
    }

    pub fn no(config: &PlyConfig, args: &NoArgs) -> Result<()> {
        let mut document = document::read::<Application>(Path::new(&args.path))?;

        document.record.stages.push(Stage {
            start_time: Utc::now(),
            deadline: None,
            name: None,
            stage_type: StageType::Rejected,
        });

        document
            .write(&config.data_dir)
            .context("failed to write new stage to document")?;

        Ok(())
    }

    pub fn edit(config: &PlyConfig, args: &EditArgs) -> Result<()> {
        // fetch all applications for the company that aren't ghosted

        // prompt to select application

        // open the document
        Err(Error::msg("unimplemented!"))
    }
}

fn main() -> Result<()> {
    let args = Ply::parse();

    // TODO: take in data directory, within a ply.toml
    let config = config::config();
    match args.command {
        Commands::To(args) => ply::to(&config, &args).context("failed to process `to` command"),
        Commands::Yes(args) => ply::yes(&config, &args).context("failed to process `yes` command"),
        Commands::No(args) => ply::no(&config, &args).context("failed to process `no` command"),
        Commands::Edit(args) => {
            ply::edit(&config, &args).context("failed to process `edit` command")
        }
    }
}
