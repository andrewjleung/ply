use std::collections::HashSet;
use std::fs;
use std::fs::DirBuilder;
use std::os;

use anyhow::{Context, Result};
use camino::Utf8Path as Path;
use camino::Utf8PathBuf as PathBuf;
use clap::{Args, Parser, Subcommand};

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
mod scraper;

struct PlyConfig {
    data_dir: PathBuf,
    days_to_ghost: u16,
}

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

    /// If set, open the document for this new application in your configured EDITOR
    editor: bool,
}

#[derive(Args)]
struct YesArgs {
    /// The company name of the job
    company: Option<String>,
}

#[derive(Args)]
struct NoArgs {
    /// The company name of the job
    company: Option<String>,
}

#[derive(Args)]
struct EditArgs {
    /// The company name of the job
    company: Option<String>,
}

mod ply {
    use crate::{EditArgs, NoArgs, PlyConfig, ToArgs, YesArgs, application::Application};
    use anyhow::{Error, Result};

    pub fn to(config: &PlyConfig, args: &ToArgs) -> Result<()> {
        // fetch the application

        // create a document

        // open the document if requested
        Err(Error::msg("unimplemented!"))
    }

    pub fn yes(config: &PlyConfig, args: &YesArgs) -> Result<()> {
        // fetch all applications for the company that aren't ghosted

        // prompt to select application

        // prompt for next steps

        // write to the document

        // open the document if requested
        Err(Error::msg("unimplemented!"))
    }

    pub fn no(config: &PlyConfig, args: &NoArgs) -> Result<()> {
        // fetch all applications for the company that aren't ghosted

        // prompt to select application

        // write to the document
        Err(Error::msg("unimplemented!"))
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
    let config = PlyConfig {
        data_dir: Path::new("data").to_path_buf(),
        days_to_ghost: 90,
    };

    DirBuilder::new().create(&config.data_dir).context(format!(
        "failed to create data directory at {}",
        config.data_dir
    ))?;

    match args.command {
        Commands::To(args) => ply::to(&config, &args).context("failed to process `to` command"),
        Commands::Yes(args) => ply::yes(&config, &args).context("failed to process `yes` command"),
        Commands::No(args) => ply::no(&config, &args).context("failed to process `no` command"),
        Commands::Edit(args) => {
            ply::edit(&config, &args).context("failed to process `edit` command")
        }
    }
}
