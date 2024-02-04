# Tataki

Tataki is a command-line tool designed primarily for detecting file formats in the bio-science field with the following features:

- Supports various **file formats mainly used in bio-science**
  - bam
  - bcf
  - bed
  - cram
  - fasta
  - fastq
  - gff3
  - gtf
  - sam
  - vcf
  - will be added in the future
- Allows for the invocation of a [**CWL document**](https://www.commonwl.org/) and enables users to define their own complex criteria for detection.
- Can target both local files and remote URLs
- Compatible with [EDAM ontology](https://edamontology.org/page)

## Installation

A single binary is available for Linux x86_64.

```shell
curl -fsSL -O https://github.com/sapporo-wes/tataki/releases/latest/download/tataki
chmod +x ./tataki
./tataki -V
```

Or, you could clone the repository, then run `cargo build`.

## Usage

Specify the paths of the files as arguments to `tataki`. Both local file path and remote URL are supported.

```shell
tataki <FILE|URL>...
```

For more details:

```shell
$ tataki --help
Usage: tataki [OPTIONS] [FILE|URL]...

Arguments:
  [FILE|URL]...  Path to the file

Options:
  -o, --output <FILE>     Path to the output file [default: stdout]
  -f <OUTPUT_FORMAT>      [default: csv] [possible values: yaml, tsv, csv, json]
      --cache-dir <DIR>   Specify the directory in which to create a temporary directory. If this option is not provided, a temporary directory will be created in the default system temporary directory (/tmp)
  -c, --conf <FILE>       Specify the tataki configuration file. If this option is not provided, the default configuration will be used. The option `--dry-run` shows the default configuration file
      --dry-run           Output the configuration file in yaml format and exit the program. If `--conf` option is not provided, the default configuration file will be shown
  -v, --verbose           Show verbose log messages
  -q, --quiet             Suppress all log messages
  -h, --help              Print help
  -V, --version           Print version

Version: 0.2.1
```

### Determining Formats in Your Preferred Order

Using the `-c|--conf=<FILE>` option allows you to change the order or set the file formats to use for determination.

The configuration file is in YAML format. Please refer to the default configuration file for the schema.

```yaml
order:
  - bam
  - bcf
  - bed
  - cram
  - fasta
  - fastq
  - gff3
  - gtf
  - sam
  - vcf
```

### Executing a CWL Document with External Extension Mode

Tataki can also be used to execute a CWL document with external extension mode. This is useful when determining file formats that are not supported in pre-built mode or when you want to perform complex detections.

This mode is dependent on Docker, so please ensure that 'docker' is in your PATH.

Here are the steps to execute a CWL document with external extension mode.

1. Prepare a CWL document
2. Specify the CWL document in the configuration file
3. Execute `tataki`.

#### Preparation of CWL Document

The CWL document must be prepared in advance. The following is an example of a CWL document that executes `samtools view`.

`edam_Id` and `label` are the two required fields for the CWL document. Both must be listed in the `tataki` prefix listed in the `$namespaces` section of the document.

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

#### Configuration File

Insert a path to the CWL document in [the configuration file](#determining-formats-in-your-preferred-order). This example shown below executes the CWL document followed by SAM and BAM format detection.

```yaml
order:
  - ./path/to/cwl_document.cwl
  - sam
  - bam
```

## License

The contents of this deposit are basically licensed under the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). See the [LICENSE](https://github.com/sapporo-wes/tataki/blob/main/LICENSE).
However, the following files are licensed under Creative Commons Attribution Share Alike 4.0 International (<https://spdx.org/licenses/CC-BY-SA-4.0.html>).

- `./src/EDAM_1.25.id_label.csv`
  - Source: <https://github.com/edamontology/edamontology/releases/download/1.25/EDAM_1.25.csv>
  - Removed the lines not related to 'format' and the columns other than 'Preferred Label' and 'Class ID'
