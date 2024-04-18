use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::module::{InvokeOptions, ModuleResult};
use crate::parser::Parser;

pub struct Bed;

impl Parser for Bed {
    fn determine_from_path(
        &self,
        input_path: &Path,
        options: &InvokeOptions,
    ) -> anyhow::Result<ModuleResult> {
        let mut reader = File::open(input_path)
            .map(BufReader::new)
            .map(noodles::bed::Reader::new)?;

        for (count, result) in reader.records::<3>().enumerate() {
            #[allow(unused_variables)]
            let record = result?;

            // If the tidy option is not set, the number of lines to read is limited to num_records.
            if !options.tidy && count + 2 > options.num_records {
                break;
            }
        }

        Ok(ModuleResult::with_result(
            Some("BED".to_string()),
            Some("http://edamontology.org/format_3003".to_string()),
        ))
    }
}
