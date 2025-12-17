use std::path::Path;

use crate::module::{InvokeOptions, ModuleResult};
use crate::parser::Parser;

pub struct Cram;

impl Parser for Cram {
    fn determine_from_path(
        &self,
        input_path: &Path,
        #[allow(unused_variables)] options: &InvokeOptions,
    ) -> anyhow::Result<ModuleResult> {
        let mut reader = noodles::cram::reader::Builder::default().build_from_path(input_path)?;

        // Check for the CRAM magic number and read its SAM header.
        #[allow(unused_variables)]
        let header = reader.read_header()?;

        // TODO: need reference sequence handling
        /*
        for (count, result) in reader.records(&header).enumerate() {
            #[allow(unused_variables)]
            let record = result?;

            // If the tidy option is not set, the number of lines to read is limited to num_records.
            if !options.tidy && count + 2 > options.num_records {
                break;
            }
        }
        */

        Ok(ModuleResult::with_result(
            Some("CRAM".to_string()),
            Some("http://edamontology.org/format_3462".to_string()),
        ))
    }
}
