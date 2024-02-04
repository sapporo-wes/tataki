use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::module::ModuleResult;
use crate::parser::Parser;

pub struct Gtf;

impl Parser for Gtf {
    fn determine(&self, input_path: &Path) -> anyhow::Result<ModuleResult> {
        let mut reader = File::open(input_path)
            .map(BufReader::new)
            .map(noodles::gtf::Reader::new)?;

        for result in reader.records() {
            #[allow(unused_variables)]
            let record = result?;
        }

        Ok(ModuleResult::with_result(
            Some("GTF".to_string()),
            Some("http://edamontology.org/format_2306".to_string()),
        ))
    }
}
