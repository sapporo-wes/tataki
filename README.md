# Tataki

Tataki is a command-line tool designed primarily for detecting file formats in the bioinformatics field. The tool comes with the following features:

- Supports various **file formats mainly used in bioinformatics**
  - Bioinformatics file formats
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
  - Compression formats
    - gzip
    - bzip2
  - will be added in the future
- Allows for the invocation of a [**CWL document**](https://www.commonwl.org/) and enables users to define their own complex criteria for detection.
- Can target local files, remote URLs and standard input
- Compatible with [EDAM ontology](https://edamontology.org/page)

## Installation

A single binary is available for Linux x86_64.

```shell
curl -fsSL -o ./tataki https://github.com/sapporo-wes/tataki/releases/latest/download/tataki-$(uname -m)
chmod +x ./tataki
./tataki --help
```

Or, you can run tataki using Docker.

```shell
docker run --rm -v $PWD:$PWD -w $PWD ghcr.io/sapporo-wes/tataki:latest --help
```

In case you want to execute the CWL document with external extension mode, please make sure to mount `docker.sock`, `/tmp` and any other necessary directories.

```shell
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock -v /tmp:/tmp -v $PWD:$PWD -w $PWD ghcr.io/sapporo-wes/tataki:latest --help
```

### Quick Start

Determine the file format of a local file. By default, tataki checks the first `--num-records` records of the input:

```shell
$ tataki path/to/unknown/file.txt -q
File Path,Edam ID,Label,Decompressed ID,Decompressed Label
path/to/unknown/file.txt,http://edamontology.org/format_2572,BAM,,
```

Determine the file format of remote file, and output result in YAML format:

```shell
$ tataki https://path/to/unknown/file.txt  -q -f yaml
https://path/to/unknown/file.txt:
  label: GZIP format
  id: http://edamontology.org/format_3989
  decompressed:
    id: http://edamontology.org/format_1930
    label: FASTQ
```

## Usage

Specify the paths of the files as arguments to `tataki`. Local file path, remote URL and standard input (`-`) are supported.

```shell
tataki <FILE|URL|'-'>...
```

For more details:

```shell
$ tataki --help
Usage: tataki [OPTIONS] [FILE|URL|'-']...

Arguments:
  [FILE|URL|'-']...  Path to the file, URL, or "-" to read from standard input. Multiple inputs can be specified

Options:
  -o, --output <FILE>              Path to the output file [default: stdout]
  -f <OUTPUT_FORMAT>               [default: csv] [possible values: yaml, tsv, csv, json]
  -C, --cache-dir <DIR>            Specify the directory in which to create a temporary directory. If this option is not provided, a temporary directory will be created in the default system temporary directory (/tmp)
  -c, --conf <FILE>                Specify the tataki configuration file. If this option is not provided, the default configuration will be used. The option `--dry-run` shows the default configuration file
  -t, --tidy                       Attempt to read the whole lines from the input files
      --no-decompress              Do not try to decompress the input file when detecting the file format
  -n, --num-records <NUM_RECORDS>  Number of records to read from the input file. Recommened to set it to a multiple of 4 to prevent false negatives. Conflicts with `--tidy` option [default: 100000]
      --dry-run                    Output the configuration file in yaml format and exit the program. If `--conf` option is not provided, the default configuration file will be shown
  -v, --verbose                    Show verbose log messages
  -q, --quiet                      Suppress all log messages
  -h, --help                       Print help
  -V, --version                    Print version

Version: v0.4.0
```

## Detailed Usage

Table of Contents

- [Reading from Standard Input](#reading-from-standard-input)
- [Changing the Number of Records to Read](#changing-the-number-of-records-to-read)
  - [Reading the Whole Lines from the Input](#reading-the-whole-lines-from-the-input)
- [Handling Compressed Files](#handling-compressed-files)
  - [BGZF Compressed Files](#bgzf-compressed-files)
- [Determining Formats in Your Preferred Order](#determining-formats-in-your-preferred-order)
- [Executing a CWL Document with External Extension Mode](#executing-a-cwl-document-with-external-extension-mode)
  - [1. Prepare a CWL Document](#1-prepare-a-cwl-document)
  - [2. Add Path to Configuration File](#2-add-path-to-configuration-file)
  - [3. Execute Tataki with `--tidy` Option](#3-execute-tataki-with---tidy-option)

### Reading from Standard Input

Read from standard input by specifying `-` as the file path.

```shell
cat <FILE> | tataki -
```

### Changing the Number of Records to Read

By default, Tataki reads the first 100,000 records of the input. You can change this number by using the `-n|--num-records=<NUM_RECORDS>` option.

```shell
tataki <FILE|URL|'-'> -n 1000
```

#### Reading the Whole Lines from the Input

By using the `-t|--tidy` option, Tataki attempts to read the whole lines from the input. This option helps when the input is truncated or its end is corrupted, and it avoids misidentifying the file formats of corrupted files

```shell
tataki <FILE|URL> -t
```

### Handling Compressed Files

Tataki attempts to automatically decompresses the input when detecting the file format. Currently, gzip and bzip2 are supported.

```shell
$ tataki foo.fastq.gz  -q -f yaml
foo.fastq.gz:
  label: GZIP format
  id: http://edamontology.org/format_3989
  decompressed:
    id: http://edamontology.org/format_1930
    label: FASTQ
```

If you want to disable the decompression, use the `--no-decompress` option.

```shell
$ tataki foo.fastq.gz  -q -f yaml --no-decompress
foo.fastq.gz:
  label: GZIP format
  id: http://edamontology.org/format_3989
  decompressed:
    id: null
    label: null
```

#### BGZF Compressed Files

BGZF compressed files, such as BCF, BAM, or anything compressed with BGZF, is handled slight differently. When BGZF files are given as input, Tataki does not attempt to decompress them and pass them directly to the parsers.

```shell
$ tataki foo.bam  -q -f yaml
foo.bam:
  label: BAM
  id: http://edamontology.org/format_2572
  decompressed:
    label: null
    id: null
```

### Determining Formats in Your Preferred Order

Using the `-c|--conf=<FILE>` option allows you to change the order or set the file formats to use for determination.

The configuration file is in YAML format. Please refer to the default configuration shown below for the schema. See [Specify the CWL document in the configuration file](#2-add-path-to-configuration-file) for details on how to specify the CWL document.

The default configuration can be achieved by using the `--dry-run` option.

```yaml
# $ tataki --dry-run
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

Tataki can also be used to execute a CWL document with external extension mode. This is useful when determining file formats that are not supported in pre-built mode or when you want to re-use the existing software to parse the input file.

This mode is dependent on Docker, so please ensure that 'docker' is in your PATH.

Here are the steps to execute a CWL document with external extension mode.

1. [Prepare a CWL document](#1-prepare-a-cwl-document)
2. [Add Path to Configuration File](#2-add-path-to-configuration-file)
3. [Execute Tataki with `--tidy` Option](#3-execute-tataki-with---tidy-option)

#### 1. Prepare a CWL Document

Tataki accepts a CWL document in a specific format. The following is an example of a CWL document that executes `samtools view`.

`edam_id` and `label` are the two required fields for the CWL document. Both must have `tataki` prefix which is listed in the `$namespaces` section of the document.

```cwl
cwlVersion: v1.2
class: CommandLineTool

requirements:
  DockerRequirement:
    dockerPull: quay.io/biocontainers/samtools:1.18--h50ea8bc_1
  InlineJavascriptRequirement: {}

baseCommand: [samtools, view]

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

#### 2. Add Path to Configuration File

Insert a path to the CWL document in [the configuration file](#determining-formats-in-your-preferred-order). This example shown below executes the CWL document followed by SAM and BAM format detection.

```yaml
order:
  - ./path/to/cwl_document.cwl
  - sam
  - bam
```

#### 3. Execute Tataki with `--tidy` Option

And then, execute `tataki` with the `-c|--conf=<FILE>` option. Remember to use the `--tidy` when executing the CWL document because the whole lines are required for the tool in CWL document to parse.

```shell
tataki <FILE|URL|`-`> -c <CONFIG_FILE> --tidy
```

Also, consider using the `--no-decompress` option when you prefer to pass the input without decompression.

## Potentially Unexpected Behaviors

These are the tricky cases where the result of tataki may not be as expected. Please see [issue #6](https://github.com/sapporo-wes/tataki/issues/6) for the examples of these cases. If you encounter any unusual behavior like these examples, please consider posting to [issue #6](https://github.com/sapporo-wes/tataki/issues/6).

- Files with only header lines

Tataki will output the file as the first format which its spec for header lines matches in the order of the configuration file. If you are running tataki with the default configuration file, and the input file uses `#` as the comment delimiter, the file will be detected as a BED file.

- Gzipped binary files

Gzipped binary files, such as `*.bam.gz`, is not suppored by tataki currently. It will fail with the following error message. `Error: stream did not contain valid UTF-8`

- BGZF format

As shown in [BGZF Compressed Files](#bgzf-compressed-files), BGZF compressed files are not decompressed by tataki, and treated as is. Please be aware of this when parsing BGZF compressed files that have a `.gz` file extension, such as `*.vcf.gz`.

```shell
$ tataki SAMPLE_01.pass.vcf.gz --yaml
SAMPLE_01.pass.vcf.gz:
  id: http://edamontology.org/format_3016
  label: VCF
  decompressed:
    label: null
    id: null
```

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to add a module to the two modes and submit a pull request to us.

## License

The contents of this deposit are basically licensed under the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). See the [LICENSE](https://github.com/sapporo-wes/tataki/blob/main/LICENSE).
However, the following files are licensed under Creative Commons Attribution Share Alike 4.0 International (<https://spdx.org/licenses/CC-BY-SA-4.0.html>).

- `./src/EDAM_1.25.id_label.csv`
  - Source: <https://github.com/edamontology/edamontology/releases/download/1.25/EDAM_1.25.csv>
  - Removed the lines not related to 'format' and the columns other than 'Preferred Label' and 'Class ID'
