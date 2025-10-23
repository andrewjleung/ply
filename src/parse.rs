use anyhow::Result;

pub mod ashby;
pub mod google;
pub mod greenhouse;
pub mod hiringcafe;
pub mod meta;
pub mod salary;

pub trait Parse<Parsable, Parsed>
where
    Parsed: Sized,
{
    fn parse(p: Parsable) -> Result<Option<Parsed>>;
}
