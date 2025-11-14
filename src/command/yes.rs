use crate::{application, command::Run, config, document};
use anyhow::{Context, Result};
use camino::Utf8Path as Path;
use clap::Args;

#[derive(Args)]
pub struct Yes {
    /// The path to the application document
    pub path: String,

    /// The next stage
    #[arg(value_enum)]
    pub next_stage: application::StageType,

    /// The next stage deadline, this may be a date/timestamp or a natural language string e.g. "in 1 week"
    pub deadline: Option<String>,
}

impl Run for Yes {
    fn run(&self, config: &config::PlyConfig) -> Result<()> {
        let mut document = document::read::<application::Application>(Path::new(&self.path))?;
        let now = chrono::Utc::now();
        let deadline = match &self.deadline {
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

        document.record.stages.push(application::Stage {
            start_time: now,
            deadline,
            name: None,
            stage_type: self.next_stage,
        });

        document
            .write(&config.data_dir)
            .context("failed to write new stage to document")?;

        println!(
            "application for '{}' marked as moving forward to the next stage ({})",
            document.record.pretty_print(),
            self.next_stage
        );

        Ok(())
    }
}
