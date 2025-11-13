use camino::Utf8Path as Path;
use camino::Utf8PathBuf as PathBuf;
use std::{collections::BTreeSet, fs};

use crate::document::read;
use crate::{
    PlyConfig,
    application::Application,
    command::Run,
    document::{self, Document},
};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct List {
    #[command(subcommand)]
    command: ListCommand,
}

#[derive(Subcommand)]
pub enum ListCommand {
    /// List job applications
    Applications(Applications),

    /// List companies you've applied to
    Companies(Companies),
}

#[derive(Args)]
pub struct Applications {
    /// Only list applications that are not in a terminal state like accepted or rejected
    #[arg(short, long)]
    active: bool,

    /// Only list applications that are past the initial 'Applied' stage and not in a terminal state like accepted or rejected
    #[arg(short, long)]
    interviewing: bool,
}

#[derive(Args)]
pub struct Companies {}

impl Run for Applications {
    fn run(&self, config: &PlyConfig) -> Result<()> {
        let read_dir = fs::read_dir(&config.data_dir).context(format!(
            "failed to read files in data directory {}",
            &config.data_dir
        ))?;

        for entry in read_dir {
            if let Ok(entry) = entry
                && let Some(path) = Path::from_path(&entry.path())
                && path.is_file()
            {
                let maybe_doc: Option<Document<Application>> = PathBuf::try_from(entry.path())
                    .ok()
                    .and_then(|path| read(&path).ok());

                if let Some(doc) = maybe_doc {
                    if self.active && !doc.record.is_active() {
                        continue;
                    }

                    if self.interviewing && !doc.record.is_interviewing() {
                        continue;
                    }

                    println!("{}", entry.path().to_string_lossy())
                };
            }
        }

        Ok(())
    }
}

impl Run for Companies {
    fn run(&self, config: &PlyConfig) -> Result<()> {
        let read_dir = fs::read_dir(&config.data_dir).context(format!(
            "failed to read files in data directory {}",
            &config.data_dir
        ))?;

        let mut companies: BTreeSet<String> = BTreeSet::new();
        for entry in read_dir {
            if let Ok(entry) = entry
                && let Some(path) = Path::from_path(&entry.path())
                && path.is_file()
            {
                let doc = document::read::<Application>(path)
                    .context(format!("failed to read application at {}", path))?;

                companies.insert(doc.record.job.company);
            }
        }

        for company in companies {
            println!("{}", &company);
        }

        Ok(())
    }
}

impl Run for List {
    fn run(&self, config: &PlyConfig) -> Result<()> {
        match &self.command {
            ListCommand::Applications(cmd) => cmd.run(config),
            ListCommand::Companies(cmd) => cmd.run(config),
        }
    }
}
