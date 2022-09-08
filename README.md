# Tataki

Tataki is a command line tool for detecting life science data types.

Currently supports the following file types.

- bam
- fasta
- fastq
- fastq.gz
- bed

Notes: Under development and could perform poorly on larger files.

## Installation

A single binary is available (supports Linux only):

```shell
curl -fsSL -O https://github.com/suecharo/tataki/releases/download/0.1.0/tataki
chmod +x ./tataki
./tataki -h
```

Or, you could clone the repository, then run `cargo build`.


## Example

```
$ tataki bed12.bed
bed12.bed: 12 column BED file
$ tataki fastq01.fq.gz 
fastq01.fq.gz: gzip compressed fastq file
```

## Todo

- add support for more file types, such as .sam, .vcf, .gtf, etc.
- add support for EDAM ontology.
- implement fast mode with which the tool could perform well on larger files.
## License

[Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). See the [LICENSE](https://github.com/suecharo/tataki/blob/main/LICENSE).
