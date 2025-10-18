use crate::{job::SalaryRange, parse::Parse};
use anyhow::{Context, Result, anyhow};
use regex::{Captures, Regex};

const DEFAULT_BOUND_UNIT: &str = "year";
const WORKING_HOURS_PER_YEAR: f64 = 2080.0;

impl Parse<()> for SalaryRange {
    fn parse_with_config(s: &str, _config: &()) -> Result<Option<Self>>
    where
        Self: Sized,
    {
        let re = Regex::new(
            r#"(?xi)
            \$ \s* (?P<lower>\d+(?:,\d{3})?(?:\.\d+)?k?)
            (?: \s* \/ \s* (?P<lower_unit>hr|hour|year))?
            (?:
            \s* (?:to|-) \s*
            \$ \s* (?P<upper>\d+(?:,\d{3})?(?:\.\d+)?k?)
            (?: \s* \/ \s* (?P<upper_unit>hr|hour|year))?
            )?
"#,
        )
        .expect("failed to compile salary range regex");

        re.captures(s)
            .and_then(|captures| parse_salary_range_from_captures(&captures).transpose())
            .transpose()
    }
}

fn parse_salary_range_from_captures(captures: &Captures) -> Result<Option<SalaryRange>> {
    let maybe_lower = bound_from_captures(captures, "lower", "lower_unit")
        .context("failed to parse salary range lower bound")?;

    let maybe_upper = bound_from_captures(captures, "upper", "upper_unit")
        .context("failed to parse salary range upper bound")?;

    SalaryRange::try_from_maybe_bounds(maybe_lower, maybe_upper)
}

fn bound_from_captures(captures: &Captures, name: &str, unit_name: &str) -> Result<Option<u32>> {
    let value = captures.name(name).map(|m| m.as_str());
    let unit = unit_from_captures(captures, unit_name);
    value.map(|v| parse_yearly_bound(v, &unit)).transpose()
}

fn unit_from_captures(captures: &Captures, name: &str) -> String {
    captures
        .name(name)
        .map_or(DEFAULT_BOUND_UNIT.to_owned(), |m| m.as_str().to_owned())
}

pub fn parse_yearly_bound(value: &str, unit: &str) -> Result<u32> {
    let value = value
        .replace(",", "")
        .replace("k", "000")
        .parse::<f64>()
        .context(format!(
            "failed to parse bound with value {value} and unit {unit}"
        ))?;

    Ok(match unit {
        "hour" | "hr" => value * WORKING_HOURS_PER_YEAR,
        "year" => value,
        _ => {
            return Err(anyhow!(
                "unrecognized unit '{unit}' while parsing salary range bound"
            ));
        }
    }
    .round() as u32)
}
