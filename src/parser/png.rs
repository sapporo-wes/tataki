use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::bail;

use crate::module::{InvokeOptions, ModuleResult};
use crate::parser::Parser;

pub struct Png;

impl Parser for Png {
    fn determine_from_path(
        &self,
        input_path: &Path,
        _options: &InvokeOptions,
    ) -> anyhow::Result<ModuleResult> {
        const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let mut buf = [0u8; 8];
        File::open(input_path)?.read_exact(&mut buf)?;
        if buf != PNG_MAGIC {
            bail!("Not a PNG file: invalid magic bytes");
        }
        Ok(ModuleResult::with_result(
            Some("PNG".to_string()),
            Some("http://edamontology.org/format_3603".to_string()),
        ))
    }
}
