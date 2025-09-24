use camino::Utf8Path as Path;
use std::{collections::BTreeSet, fs};

use crate::{PlyConfig, application::Application, command::Run, document};
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
    /// Only list applications that are not terminal, i.e. accepted or rejected
    #[arg(short, long)]
    active: bool,
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
                let doc = document::read::<Application>(path)
                    .context(format!("failed to read application at {}", path))?;

                if !self.active || doc.record.is_active() {
                    println!("{}", entry.path().to_string_lossy())
                }
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
