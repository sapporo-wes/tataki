use std::path::Path;

use crate::module::ModuleResult;
use crate::parser::Parser;

pub struct Sam;

impl Parser for Sam {
    fn determine(&self, input_path: &Path) -> anyhow::Result<ModuleResult> {
        let mut reader = noodles::sam::reader::Builder::default().build_from_path(input_path)?;
        let header = reader.read_header()?;
        for result in reader.records(&header) {
            #[allow(unused_variables)]
            let record = result?;
        }

        Ok(ModuleResult::with_result(
            Some("SAM".to_string()),
            Some("http://edamontology.org/format_2573".to_string()),
        ))
    }
}
