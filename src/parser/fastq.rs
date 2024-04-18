use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::module::{InvokeOptions, ModuleResult};
use crate::parser::Parser;

pub struct Fastq;

impl Parser for Fastq {
    fn determine_from_path(
        &self,
        input_path: &Path,
        options: &InvokeOptions,
    ) -> anyhow::Result<ModuleResult> {
        let mut reader = File::open(input_path)
            .map(BufReader::new)
            .map(noodles::fastq::Reader::new)?;

        for (count, result) in reader.records().enumerate() {
            #[allow(unused_variables)]
            let record = result?;

            // If the tidy option is not set, the number of lines to read is limited to num_lines.
            if !options.tidy && count >= options.num_lines {
                break;
            }
        }

        Ok(ModuleResult::with_result(
            Some("FASTQ".to_string()),
            Some("http://edamontology.org/format_1930".to_string()),
        ))
    }
}
