use crate::{
    application::{Application, Stage, StageType},
    command::Run,
    config::PlyConfig,
    document,
};
use anyhow::{Context, Result};
use camino::Utf8Path as Path;
use chrono::Utc;
use clap::Args;

#[derive(Args)]
pub struct No {
    /// The path to the application document
    pub path: String,
}

impl Run for No {
    fn run(&self, config: &PlyConfig) -> Result<()> {
        let mut document = document::read::<Application>(Path::new(&self.path))?;

        if !document.record.is_active()
            && let Some(stage) = document.record.current_stage()
        {
            println!(
                "application for {} is already {}",
                document.record.pretty_print(),
                stage.stage_type,
            );
            return Ok(());
        }

        document.record.stages.push(Stage {
            start_time: Utc::now(),
            deadline: None,
            name: None,
            stage_type: StageType::Rejected,
        });

        document
            .write(&config.data_dir)
            .context("failed to write new stage to document")?;

        println!(
            "application for {} marked as rejected",
            document.record.pretty_print()
        );

        Ok(())
    }
}
