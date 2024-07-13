# Contrubuting Guidelines

Thank you for considering contributing to [tataki](https://github.com/sapporo-wes/tataki)! Here are some guidelines to help you get started.

## I want to add a module to the standalone mode

A [template for a module](src/parser/template.rs) is available in the [parser](src/parser/) directory.

1. Copy the template to a new file with a name that describes the format you are going to parse.
2. Rename the `Template` struct to the name of the format you are going to parse.
3. Implement the `determine_from_path` method of the `Parser` trait.
4. Add a `mod` statement to [parser.rs](src/parser.rs) so that `tataki` recognizes your module, and also add a branch to the `match` statement in the `from_str_to_parser` function.
5. Write a test for the module. An input file for the test should be placed in the [tests/inputs](tests/inputs/) directory. If the size of the input file is large, please use Zenodo.

### Implementing the `determine_from_path` method

This method determines if an input is in a format that your module can interpret.

```rs
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
            let line = line?;

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
```

#### Arguments

- `input_path`: The path to the input file.
- `options`: The options passed to the parser. `options.tidy` and `options.num_records` is used to control the number of lines to read from the input. `options.no_decompress` is irrelevant here.

```rs
pub struct InvokeOptions {
    /// Read full content of the input or not
    pub tidy: bool,
    /// Irrevant for `determine_from_path` method
    pub no_decompress: bool,
    /// Number of records to read
    pub num_records: usize,
}
```

#### Return Values

- If the parser can successfully interpret the file, return `Ok(ModuleResult)`. Use `ModuleResult::with_result()` to construct the `ModuleResult`.
  - `label`: EDAM Preferred Label
  - `id`: EDAM Class ID

```rs
// example of successfull return 
Ok(ModuleResult::with_result(
    Some("BAM".to_string()),
    Some("http://edamontology.org/format_2572".to_string()),
))
```

- If the parser fails, return `Err(anyhow::Error)`, including an error message that specifies the reasons why the parser cannot process the file.

```rs
// example of failure return
return Err(anyhow::anyhow!("The input is missing the required column."));
```

## I want to add a CWL document to the external extension mode

1. Create a CWL document under the [`cwl`](cwl/) directory.
2. Create a pull request.

Please make sure that your CWL document has the following:

- `edam_id` and `label`: Both must have `tataki` prefix which is listed in the `$namespaces` section.

Example of a CWL document:

```cwl
cwlVersion: v1.2
class: CommandLineTool

requirements:
  DockerRequirement:
    dockerPull: quay.io/biocontainers/samtools:1.18--h50ea8bc_1
  InlineJavascriptRequirement: {}

baseCommand: [samtools, head]

successCodes: [0, 139]

inputs:
  input_file:
    type: File
    inputBinding:
      position: 1

outputs: {}

$namespaces:
  tataki: https://github.com/sapporo-wes/tataki
  
tataki:edam_id: http://edamontology.org/format_2573
tataki:label: SAM
```

