use std::{collections::HashSet, fs};

use crate::{
    PlyConfig,
    application::Application,
    command::Run,
    document::{Document, read},
};
use anyhow::{Context, Result};
use camino::Utf8PathBuf as PathBuf;
use clap::Args;

#[derive(Args)]
pub struct Cycles {}

impl Run for Cycles {
    fn run(&self, config: &PlyConfig) -> Result<()> {
        let entries = fs::read_dir(&config.data_dir).context(format!(
            "failed to access data directory at {}",
            &config.data_dir
        ))?;

        let mut cycles: HashSet<String> = HashSet::new();

        for entry in entries.flatten() {
            if entry.path().is_file() {
                let maybe_doc: Option<Document<Application>> = PathBuf::try_from(entry.path())
                    .ok()
                    .and_then(|path| read(&path).ok());

                if let Some(doc) = maybe_doc
                    && let Some(cycle) = doc.record.cycle
                {
                    cycles.insert(cycle);
                };
            }
        }

        let mut cycles = cycles.iter().collect::<Vec<_>>();
        cycles.sort();

        for cycle in cycles {
            println!("{}", &cycle);
        }

        Ok(())
    }
}
