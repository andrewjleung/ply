use crate::{command::Run, config::PlyConfig};
use anyhow::Result;

mod application;
mod command;
mod config;
mod data;
mod document;
mod fetch;
mod job;
mod job_listing;
mod parse;
mod scrape;

fn main() -> Result<()> {
    let config = config::config();
    command::parse().run(&config)
}
