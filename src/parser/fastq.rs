use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::parser::Parser;
use crate::run::ModuleResult;

pub struct Fastq;

impl Parser for Fastq {
    fn determine(&self, input_path: &Path) -> anyhow::Result<ModuleResult> {
        let mut reader = File::open(input_path)
            .map(BufReader::new)
            .map(noodles::fastq::Reader::new)?;

        for result in reader.records() {
            #[allow(unused_variables)]
            let record = result?;
        }

        Ok(ModuleResult::with_result(
            Some("FASTQ".to_string()),
            Some("http://edamontology.org/format_1930".to_string()),
        ))
    }
}
