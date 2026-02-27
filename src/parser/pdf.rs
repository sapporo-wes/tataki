use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::bail;

use crate::module::{InvokeOptions, ModuleResult};
use crate::parser::Parser;

pub struct Pdf;

impl Parser for Pdf {
    fn determine_from_path(
        &self,
        input_path: &Path,
        _options: &InvokeOptions,
    ) -> anyhow::Result<ModuleResult> {
        let mut buf = [0u8; 5];
        File::open(input_path)?.read_exact(&mut buf)?;
        if &buf != b"%PDF-" {
            bail!("Not a PDF file: missing %PDF- header");
        }
        Ok(ModuleResult::with_result(
            Some("PDF".to_string()),
            Some("http://edamontology.org/format_3508".to_string()),
        ))
    }
}
