use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::bail;

use crate::module::{InvokeOptions, ModuleResult};
use crate::parser::Parser;

pub struct Svg;

impl Parser for Svg {
    fn determine_from_path(
        &self,
        input_path: &Path,
        _options: &InvokeOptions,
    ) -> anyhow::Result<ModuleResult> {
        let mut buf = [0u8; 1024];
        let mut file = File::open(input_path)?;
        let n = file.read(&mut buf)?;
        let text = std::str::from_utf8(&buf[..n]).unwrap_or("").to_ascii_lowercase();
        if !text.contains("<svg") {
            bail!("Not an SVG file: missing <svg element");
        }
        Ok(ModuleResult::with_result(
            Some("SVG".to_string()),
            Some("http://edamontology.org/format_3604".to_string()),
        ))
    }
}
