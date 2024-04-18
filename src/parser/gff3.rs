use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::module::{InvokeOptions, ModuleResult};
use crate::parser::Parser;

pub struct Gff3;

impl Parser for Gff3 {
    fn determine_from_path(
        &self,
        input_path: &Path,
        options: &InvokeOptions,
    ) -> anyhow::Result<ModuleResult> {
        let mut reader = File::open(input_path)
            .map(BufReader::new)
            .map(noodles::gff::Reader::new)?;

        for (count, result) in reader.records().enumerate() {
            #[allow(unused_variables)]
            let record = result?;

            // If the tidy option is not set, the number of lines to read is limited to num_records.
            if !options.tidy && count + 2 > options.num_records {
                break;
            }
        }

        Ok(ModuleResult::with_result(
            Some("GFF3".to_string()),
            Some("http://edamontology.org/format_1975".to_string()),
        ))
    }
}
