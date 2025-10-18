use anyhow::Result;

pub mod salary;

pub trait Parse<Config>
where
    Config: Default,
    Self: Sized,
{
    fn parse_with_config(s: &str, config: &Config) -> Result<Option<Self>>;

    fn parse(s: &str) -> Result<Option<Self>> {
        Self::parse_with_config(s, &Config::default())
    }
}
