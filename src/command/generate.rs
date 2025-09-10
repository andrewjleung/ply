use std::io::stdout;

use crate::{
    PlyConfig,
    command::{Ply, Run},
};
use anyhow::Result;
use clap::{Args, CommandFactory};
use clap_complete::{Shell, generate};

#[derive(Args)]
pub struct Generate {
    #[arg(value_enum)]
    /// The shell to generate completions for
    shell: Shell,
}

impl Run for Generate {
    fn run(&self, _config: &PlyConfig) -> Result<()> {
        generate(self.shell, &mut Ply::command(), "ply", &mut stdout());
        Ok(())
    }
}
