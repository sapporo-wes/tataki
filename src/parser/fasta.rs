use std::path::Path;

use crate::module::{InvokeOptions, ModuleResult};
use crate::parser::Parser;

pub struct Fasta;

impl Parser for Fasta {
    fn determine_from_path(
        &self,
        input_path: &Path,
        options: &InvokeOptions,
    ) -> anyhow::Result<ModuleResult> {
        let mut reader = noodles::fasta::reader::Builder.build_from_path(input_path)?;

        for (count, result) in reader.records().enumerate() {
            #[allow(unused_variables)]
            let record = result?;

            // If the tidy option is not set, the number of lines to read is limited to num_records.
            if !options.tidy && count + 2 > options.num_records {
                break;
            }
        }

        Ok(ModuleResult::with_result(
            Some("FASTA".to_string()),
            Some("http://edamontology.org/format_1929".to_string()),
        ))
    }
}
