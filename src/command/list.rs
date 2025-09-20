use camino::Utf8Path as Path;
use std::fs;

use crate::{PlyConfig, application::Application, command::Run, document};
use anyhow::{Context, Result};
use clap::Args;

#[derive(Args)]
pub struct List {
    /// Only list applications that are not terminal, i.e. accepted or rejected
    #[arg(short, long)]
    active: bool,
}

impl Run for List {
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
