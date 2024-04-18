use std::path::Path;

use crate::module::ModuleResult;
use crate::parser::Parser;

pub struct Vcf;

impl Parser for Vcf {
    fn determine_from_path(&self, input_path: &Path) -> anyhow::Result<ModuleResult> {
        let mut reader = noodles::vcf::reader::Builder::default().build_from_path(input_path)?;

        let header = reader.read_header()?;

        for result in reader.records(&header) {
            #[allow(unused_variables)]
            let record = result?;
        }

        Ok(ModuleResult::with_result(
            Some("VCF".to_string()),
            Some("http://edamontology.org/format_3016".to_string()),
        ))
    }
}
