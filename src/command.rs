use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::{
    command::{
        config::Config, cycles::Cycles, data_directory::DataDirectory, generate::Generate,
        list::List, no::No, to::To, yes::Yes,
    },
    config::PlyConfig,
};

mod config;
mod cycles;
mod data_directory;
mod generate;
mod list;
mod no;
mod to;
mod yes;

pub trait Run {
    fn run(&self, config: &PlyConfig) -> Result<()>;
}

#[derive(Parser)]
#[command(version, about)]
pub struct Ply {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Fetch the configured data directory
    DataDirectory(DataDirectory),

    /// Fetch the config path
    Config(Config),

    /// Fetch all cycles with applications
    Cycles(Cycles),

    /// Generate completions for this CLI
    Generate(Generate),

    /// List data about applications
    List(List),

    /// Mark an application as rejected
    No(No),

    /// Create an application to a job listing
    To(To),

    /// Mark an application moving onto the next stage
    Yes(Yes),
}

pub fn parse() -> Ply {
    Ply::parse()
}

impl Run for Ply {
    fn run(&self, config: &PlyConfig) -> Result<()> {
        match &self.command {
            Command::Config(cmd) => cmd.run(config),
            Command::Cycles(cmd) => cmd.run(config),
            Command::DataDirectory(cmd) => cmd.run(config),
            Command::Generate(cmd) => cmd.run(config),
            Command::List(cmd) => cmd.run(config),
            Command::No(cmd) => cmd.run(config),
            Command::To(cmd) => cmd.run(config),
            Command::Yes(cmd) => cmd.run(config),
        }
    }
}
