use std::{collections::HashSet, fs};

use crate::{
    PlyConfig,
    application::Application,
    command::Run,
    document::{Document, read},
};
use anyhow::{Context, Result, anyhow};
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
                let path = entry.path();
                let path = PathBuf::from_path_buf(path).map_err(|path_buf| {
                    anyhow!(
                        "path {} is not UTF-8",
                        path_buf.to_string_lossy().into_owned()
                    )
                })?;

                let application: Document<Application> =
                    read(&path).context(format!("failed to read application at {}", path))?;

                if let Some(cycle) = application.record.cycle {
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
