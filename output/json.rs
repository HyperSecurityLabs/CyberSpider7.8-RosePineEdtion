use anyhow::Result;
use serde_json;
use crate::{SpiderResult, output::OutputFormatter};

pub struct JsonFormatter;

impl OutputFormatter for JsonFormatter {
    fn format(&self, result: &SpiderResult) -> Result<String> {
        let json = serde_json::to_string_pretty(result)?;
        Ok(json)
    }
}
