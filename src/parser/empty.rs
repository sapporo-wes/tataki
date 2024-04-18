use std::fs;
use std::path::Path;

use crate::module::ModuleResult;
use crate::parser::Parser;

pub struct Empty;

impl Parser for Empty {
    // check if the file is empty or not.
    fn determine_from_path(&self, input_path: &Path) -> anyhow::Result<ModuleResult> {
        let metadata = fs::metadata(input_path)?;
        if metadata.len() == 0 {
            Ok(ModuleResult::with_result(
                Some("plain text format (unformatted)".to_string()),
                Some("http://edamontology.org/format_1964".to_string()),
            ))
        } else {
            anyhow::bail!("The file is not empty");
        }
    }
}
