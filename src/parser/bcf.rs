use std::path::Path;

use crate::parser::Parser;
use crate::run::ModuleResult;

pub struct Bcf;

impl Parser for Bcf {
    fn determine(&self, input_path: &Path) -> anyhow::Result<ModuleResult> {
        let mut reader = noodles::bcf::reader::Builder::default().build_from_path(input_path)?;

        let header = reader.read_header()?;

        for result in reader.records(&header) {
            #[allow(unused_variables)]
            let record = result?;
        }

        Ok(ModuleResult::with_result(
            Some("BCF".to_string()),
            Some("http://edamontology.org/format_3020".to_string()),
        ))
    }
}
