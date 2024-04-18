use std::path::Path;

use crate::module::{InvokeOptions, ModuleResult};
use crate::parser::Parser;

pub struct Bam;

impl Parser for Bam {
    fn determine_from_path(
        &self,
        input_path: &Path,
        options: &InvokeOptions,
    ) -> anyhow::Result<ModuleResult> {
        let mut reader = noodles::bam::reader::Builder.build_from_path(input_path)?;
        let header = reader.read_header()?;

        for (count, result) in reader.records(&header).enumerate() {
            #[allow(unused_variables)]
            let record = result?;

            // If the tidy option is not set, the number of lines to read is limited to num_records.
            if !options.tidy && count + 2 > options.num_records {
                break;
            }
        }
        Ok(ModuleResult::with_result(
            Some("BAM".to_string()),
            Some("http://edamontology.org/format_2572".to_string()),
        ))
    }
}
