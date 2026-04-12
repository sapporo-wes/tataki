use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::bail;

use crate::module::{InvokeOptions, ModuleResult};
use crate::parser::Parser;

pub struct Tsv;

impl Parser for Tsv {
    fn determine_from_path(
        &self,
        input_path: &Path,
        options: &InvokeOptions,
    ) -> anyhow::Result<ModuleResult> {
        let reader = BufReader::new(File::open(input_path)?);
        let mut column_count: Option<usize> = None;
        let mut row_count = 0usize;

        for (i, line) in reader.lines().enumerate() {
            if !options.tidy && i >= options.num_records {
                break;
            }
            let line = line?;
            // Skip comment lines and blank lines
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let cols = line.split('\t').count();
            if cols < 2 {
                bail!("Not a TSV file: line has fewer than 2 tab-separated fields");
            }
            match column_count {
                None => column_count = Some(cols),
                Some(expected) if expected != cols => {
                    bail!(
                        "Not a TSV file: inconsistent column count ({} vs {})",
                        expected,
                        cols
                    );
                }
                _ => {}
            }
            row_count += 1;
        }

        if row_count == 0 {
            bail!("Not a TSV file: no data rows found");
        }

        Ok(ModuleResult::with_result(
            Some("TSV".to_string()),
            Some("http://edamontology.org/format_3475".to_string()),
        ))
    }
}
