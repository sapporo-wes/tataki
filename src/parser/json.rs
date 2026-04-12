use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use anyhow::bail;

use crate::module::{InvokeOptions, ModuleResult};
use crate::parser::Parser;

// Maximum bytes to read for JSON validation (64 KB is enough to detect structure).
const JSON_PROBE_SIZE: u64 = 65536;

pub struct Json;

impl Parser for Json {
    fn determine_from_path(
        &self,
        input_path: &Path,
        _options: &InvokeOptions,
    ) -> anyhow::Result<ModuleResult> {
        let file = File::open(input_path)?;
        let mut limited = BufReader::new(file.take(JSON_PROBE_SIZE));
        let mut buf = Vec::new();
        limited.read_to_end(&mut buf)?;

        match serde_json::from_slice::<serde_json::Value>(&buf) {
            Ok(_) => {}
            Err(e) => {
                // An EOF error means the file was truncated at our probe limit but was
                // syntactically valid up to that point - treat as JSON.
                if e.classify() != serde_json::error::Category::Eof {
                    bail!("Not a JSON file: {}", e);
                }
            }
        }

        Ok(ModuleResult::with_result(
            Some("JSON".to_string()),
            Some("http://edamontology.org/format_3464".to_string()),
        ))
    }
}
