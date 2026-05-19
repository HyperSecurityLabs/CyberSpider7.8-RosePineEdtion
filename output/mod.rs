pub mod json;
pub mod text;

use anyhow::Result;
use crate::SpiderResult;

pub trait OutputFormatter {
    fn format(&self, result: &SpiderResult) -> Result<String>;
}
