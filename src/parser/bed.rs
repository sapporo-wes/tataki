use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::module::ModuleResult;
use crate::parser::Parser;

pub struct Bed;

impl Parser for Bed {
    fn determine_from_path(&self, input_path: &Path) -> anyhow::Result<ModuleResult> {
        let mut reader = File::open(input_path)
            .map(BufReader::new)
            .map(noodles::bed::Reader::new)?;

        for result in reader.records::<3>() {
            #[allow(unused_variables)]
            let record = result?;
        }

        Ok(ModuleResult::with_result(
            Some("BED".to_string()),
            Some("http://edamontology.org/format_3003".to_string()),
        ))
    }
}
