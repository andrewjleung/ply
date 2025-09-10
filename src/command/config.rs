use crate::{PlyConfig, command::Run, config::default_config_path};
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct Config {}

impl Run for Config {
    fn run(&self, _config: &PlyConfig) -> Result<()> {
        println!("{}", default_config_path());
        Ok(())
    }
}
