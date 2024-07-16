use std::io::BufRead;
use std::path::Path;

use crate::module::{InvokeOptions, ModuleResult};
use crate::parser::Parser;

pub struct Template;

impl Parser for Template {
    fn determine_from_path(
        &self,
        input_path: &Path,
        options: &InvokeOptions,
    ) -> anyhow::Result<ModuleResult> {
        /*
        This is a dummy implementation. Replace this with the actual algorithm to determine the file format.
        */

        let file = std::fs::File::open(input_path)?;
        let reader = std::io::BufReader::new(file);

        for (count, line) in reader.lines().enumerate() {
            let _ = line?;

            // Do something with the line here

            // in case of parser error
            if false {
                return Err(anyhow::anyhow!("The input is missing the required column."));
            }

            // If the tidy option is not set, the number of lines to read is limited to num_records. +2 is used as a buffer.
            if !options.tidy && count + 2 > options.num_records {
                break;
            }
        }

        Ok(ModuleResult::with_result(
            Some("EDAM label".to_string()),
            Some("http://edamontology.org/format_ EDAM id".to_string()),
        ))
    }
}

/*
#[test]
fn test_template() {
    let input_path = Path::new("./tests/inputs/template");
    let option = InvokeOptions {
        tidy: true,
        no_decompress: false,
        num_records: 100000,
    };

    let parser = Template;
    let module_result = parser.determine_from_path(input_path, &option).unwrap();

    assert_eq!(module_result.label(), Some(&"EDAM label".to_string()));
    assert_eq!(
        module_result.id(),
        Some(&"http://edamontology.org/format_ EDAM id".to_string())
    );
}
*/
