use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::parser::Parser;
use crate::run::ModuleResult;

pub struct Gff3;

impl Parser for Gff3 {
    fn determine(&self, input_path: &Path) -> anyhow::Result<ModuleResult> {
        let mut reader = File::open(input_path)
            .map(BufReader::new)
            .map(noodles::gff::Reader::new)?;

        for result in reader.records() {
            #[allow(unused_variables)]
            let record = result?;
        }

        Ok(ModuleResult::with_result(
            Some("GFF3".to_string()),
            Some("http://edamontology.org/format_1975".to_string()),
        ))
    }
}
