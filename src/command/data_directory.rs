use crate::{PlyConfig, command::Run};
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct DataDirectory {}

impl Run for DataDirectory {
    fn run(&self, config: &PlyConfig) -> Result<()> {
        println!("{}", config.data_dir);
        Ok(())
    }
}
