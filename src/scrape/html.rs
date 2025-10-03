use anyhow::{Context, Error, Result, anyhow};

pub trait HTMLParse<T> {
    fn select() -> Result<String>;
    fn parse(selection: &str) -> Result<T>;
}
