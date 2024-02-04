use std::path::Path;

use crate::module::ModuleResult;
use crate::parser::Parser;

pub struct Fasta;

impl Parser for Fasta {
    fn determine(&self, input_path: &Path) -> anyhow::Result<ModuleResult> {
        let mut reader = noodles::fasta::reader::Builder.build_from_path(input_path)?;

        for result in reader.records() {
            #[allow(unused_variables)]
            let record = result?;
        }

        Ok(ModuleResult::with_result(
            Some("FASTA".to_string()),
            Some("http://edamontology.org/format_1929".to_string()),
        ))
    }
}
