use crate::{command::Run, config::PlyConfig};
use anyhow::Result;

mod application;
mod command;
mod config;
mod document;
mod job;
mod scrape;

fn main() -> Result<()> {
    let config = config::config();
    command::parse().run(&config)
}
