# Contrubuting Guidelines

Thank you for considering contributing to [tataki](https://github.com/sapporo-wes/tataki)! Here are some guidelines to help you get started.

Table of Contents

- [I want to add a module to the standalone mode](#i-want-to-add-a-module-to-the-standalone-mode)
- [I want to add a CWL document to the external extension mode](#i-want-to-add-a-cwl-document-to-the-external-extension-mode)

## I want to add a module to the standalone mode

A [template for a module](src/parser/template.rs) is available in the [parser](src/parser/) directory.

Instructions:

1. Copy the template to a new file with a name that describes the format you are going to parse.
2. Rename the `Template` struct to the name of the format you are going to parse.
3. Implement the `determine_from_path` method of the `Parser` trait. Detailed instructions are provided [below](#implementing-the-determine_from_path-method).
4. Add a `mod` statement (e.g., `mod bam;`) to the top of [parser.rs](src/parser.rs) so that `tataki` recognizes your module, and also add a branch to the `match` statement in the `from_str_to_parser` function.
5. Write a test for the module. An input file for the test should be placed in the [tests/inputs](tests/inputs/) directory. If the size of the input file is large, please use Zenodo.
6. Run all tests by executing `cargo test` to ensure everything is working correctly.

### Implementing the `determine_from_path` method

`determine_from_path` is a core method of the `Parser` trait. This method determines whether the input matches the target file format. Implementing your file format parsing algorithm here.

The method reads the input file line by line from the `reader` and validates the format of each line. If a single record consists of multiple lines (e.g., a FASTQ record has 4 lines), implement logic that groups and validates these lines together as one complete record. Note that headers are not counted as records, so if the file format includes a header section, parse and validate the header first before processing records. For details on the method's [arguments](#arguments) and [return values](#return-values), refer to the sections below.

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

        // Read the header in case the format has a header.
        /*
        example_read_header()?;
         */

        // Read the input line by line and check if it matches the expected format.
        for (count, line) in reader.lines().enumerate() {
            let _line = line?;

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
- `options`: The options passed to the parser. `options.tidy` and `options.num_records` are used to control the number of lines to read from the input. `options.no_decompress` is irrelevant for this method.

```rs
pub struct InvokeOptions {
    /// Read full content of the input or not
    pub tidy: bool,
    /// Irrelevant for `determine_from_path` method
    pub no_decompress: bool,
    /// Number of records to read
    pub num_records: usize,
}
```

#### Return Values

- If the parser can successfully interpret the file, return `Ok(ModuleResult)`. Use `ModuleResult::with_result(label: Option<String>, id: Option<String>)` to construct the `ModuleResult` with the Edam ontology information.
  - `label`: EDAM Preferred Label
  - `id`: EDAM Class ID

```rs
// Example of successful return 
Ok(ModuleResult::with_result(
    Some("BAM".to_string()),
    Some("http://edamontology.org/format_2572".to_string()),
))
```

- If the parser fails, return `Err(anyhow::Error)` with an error message specifying why the parser cannot process the file.

```rs
// Example of failure return
return Err(anyhow::anyhow!("The input is missing the required column."));
```

## I want to add a CWL document to the external extension mode

A [template for a CWL document](cwl/template.cwl) is available in the [cwl](cwl/) directory.

Instructions:

1. Copy the template and create a CWL document under the [`cwl`](cwl/) directory.
2. Configure the CWL document with the docker image, base command, and Edam ontology information.

### Configuring the CWL document

The CWL document is used for input format validation. The tool specified in `baseCommand` attempts to read and parse the input file, and if it succeeds (exits with a success code), the input is considered to be in the valid format.

Please make sure that your CWL document has the following:

- `requirements.DockerRequirement.dockerPull`: The docker image that the CWL document uses.
- `baseCommand`: The base command with which the docker image is executed to parse the input.
- `edam_id` and `label`: Describe the Edam ontology information when the parse is successfull. Both must have `tataki` prefix as shown in the example below.

Example of a CWL document:

```cwl
cwlVersion: v1.2
class: CommandLineTool

# Configure docker image here
requirements:
  DockerRequirement:
    dockerPull: your_docker_image
  InlineJavascriptRequirement: {}

# Configure base command here
baseCommand: [command, to, use]

successCodes: [0, 139]

inputs:
  input_file:
    type: File
    inputBinding:
      position: 1

outputs: {}

$namespaces:
  tataki: https://github.com/sapporo-wes/tataki
  
# Configure Edam ontology information here
tataki:edam_id: http://edamontology.org/format_edam-id
tataki:label: edam-label
```
