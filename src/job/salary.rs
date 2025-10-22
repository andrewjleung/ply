use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SalaryRange {
    pub lower: u32,
    pub range: Option<u32>,
}

impl SalaryRange {
    #[allow(dead_code)]
    pub fn upper(&self) -> Option<u32> {
        self.range.map(|r| self.lower + r)
    }

    pub fn from_bounds(lower: u32, upper: u32) -> Result<Self> {
        if lower > upper {
            return Err(anyhow!(
                "invalid salary range, lower bound {lower} is greater than upper bound {upper}"
            ));
        }

        Ok(Self {
            lower,
            range: Some(upper.abs_diff(lower)),
        })
    }

    pub fn amount(amount: u32) -> Self {
        Self {
            lower: amount,
            range: None,
        }
    }

    pub fn try_from_maybe_bounds(
        maybe_lower: Option<u32>,
        maybe_upper: Option<u32>,
    ) -> Result<Option<Self>> {
        Ok(match (maybe_lower, maybe_upper) {
            (Some(lower), Some(upper)) => Some(SalaryRange::from_bounds(lower, upper)?),
            (Some(lower), None) => Some(SalaryRange::amount(lower)),
            _ => None,
        })
    }
}
