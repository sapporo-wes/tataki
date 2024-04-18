use std::path::Path;

use crate::module::ModuleResult;
use crate::parser::Parser;

pub struct Bam;

impl Parser for Bam {
    fn determine_from_path(&self, input_path: &Path) -> anyhow::Result<ModuleResult> {
        let mut reader = noodles::bam::reader::Builder.build_from_path(input_path)?;
        let header = reader.read_header()?;
        for result in reader.records(&header) {
            #[allow(unused_variables)]
            let record = result?;
        }
        Ok(ModuleResult::with_result(
            Some("BAM".to_string()),
            Some("http://edamontology.org/format_2572".to_string()),
        ))
    }
}
