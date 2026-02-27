use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::bail;

use crate::module::{InvokeOptions, ModuleResult};
use crate::parser::Parser;

pub struct Html;

impl Parser for Html {
    fn determine_from_path(
        &self,
        input_path: &Path,
        _options: &InvokeOptions,
    ) -> anyhow::Result<ModuleResult> {
        let mut buf = [0u8; 512];
        let mut file = File::open(input_path)?;
        let n = file.read(&mut buf)?;
        let text = std::str::from_utf8(&buf[..n]).unwrap_or("").to_ascii_lowercase();
        if !text.contains("<!doctype html") && !text.contains("<html") {
            bail!("Not an HTML file: missing DOCTYPE or <html element");
        }
        Ok(ModuleResult::with_result(
            Some("HTML".to_string()),
            Some("http://edamontology.org/format_2331".to_string()),
        ))
    }
}
