mod bam;
mod bcf;
mod bed;
mod cram;
mod empty;
mod fasta;
mod fastq;
mod gff3;
mod gtf;
mod sam;
mod template;
mod vcf;

use anyhow::{bail, Result};
use log::info;
use std::path::Path;

use crate::module::{InvokeOptions, ModuleResult};
use crate::source::Source;

pub trait Parser {
    /// Determines whether the provided file is in a format that this parser can interpret.
    /// If the parser successfully interprets the file, it returns `Ok(ModuleResult)`.
    /// Otherwise, it returns `Err(anyhow::Error)`, with an error message explaining why the parser cannot process the file.
    /// To construct a `ModuleResult`, use `ModuleResult::with_result()`, which requires `label` and `id` as parameters.
    ///
    /// - `id`: EDAM Class ID
    /// - `label`: EDAM Preferred Label
    fn determine_from_path(
        &self,
        input_path: &Path,
        options: &InvokeOptions,
    ) -> Result<ModuleResult>;
}

pub fn from_str_to_parser(module_name: &str) -> Result<Box<dyn Parser>> {
    let module_name = module_name.to_lowercase();
    match &module_name[..] {
        "bam" => Ok(Box::new(bam::Bam)),
        "bcf" => Ok(Box::new(bcf::Bcf)),
        "bed" => Ok(Box::new(bed::Bed)),
        "cram" => Ok(Box::new(cram::Cram)),
        "empty" => Ok(Box::new(empty::Empty)),
        "fasta" => Ok(Box::new(fasta::Fasta)),
        "fastq" => Ok(Box::new(fastq::Fastq)),
        "gff3" => Ok(Box::new(gff3::Gff3)),
        "gff" => Ok(Box::new(gff3::Gff3)),
        "gtf" => Ok(Box::new(gtf::Gtf)),
        "sam" => Ok(Box::new(sam::Sam)),
        "vcf" => Ok(Box::new(vcf::Vcf)),
        // "template" => Ok(Box::new(template::Template)),
        _ => bail!("Unsupported parser name: {}", module_name),
    }
}

// Return the result of determine() using Ok(ModuleResult), and return errors in other parts using Err.
pub fn invoke(
    module_name: &str,
    target_source: &Source,
    options: &InvokeOptions,
) -> Result<ModuleResult> {
    info!("Invoking parser {}", module_name);

    let parser = from_str_to_parser(module_name)?;

    // Convert Source to readable object
    let target_file_path = match target_source {
        Source::FilePath(target_file_path) => target_file_path,
        Source::TempFile(target_temp_file) => target_temp_file.path(),
        Source::Stdin => {
            unreachable!()
        }
        Source::Memory(_) => {
            unreachable!()
        }
    };

    Ok(parser
        .determine_from_path(target_file_path, options)
        .unwrap_or_else(|e| {
            let mut module_result = ModuleResult::with_result(None, None);
            module_result.set_is_ok(false);
            module_result.set_error_message(e.to_string());
            module_result
        }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Tests the result of invoking the parser when the `determine` (in this case `determine_from_path`) function successfully identifies the file format.
    /// This function verifies that the returned `ModuleResult` contains the expected `label` and `id`.`
    fn invoke_wrapper_determine_pass(
        module_name: &str,
        target_file_path: &Path,
        label: &str,
        id: &str,
    ) {
        let target_source = Source::FilePath(target_file_path.to_path_buf());
        let options = InvokeOptions {
            tidy: true,
            no_decompress: false,
            num_records: 100000,
        };
        let result = invoke(module_name, &target_source, &options).unwrap();

        assert_eq!(result.label(), Some(&label.to_string()));
        assert_eq!(result.id(), Some(&id.to_string()));
    }

    /// Tests the result of invoking the parser when the `determine` (in this case `determine_from_path`) function fails to identify the file format.
    /// This function verifies that the returned `ModuleResult` contains the expected error message.
    fn invoke_wrapper_determine_fail(
        module_name: &str,
        target_file_path: &Path,
        error_message: &str,
    ) {
        let target_source = Source::FilePath(target_file_path.to_path_buf());
        let options = InvokeOptions {
            tidy: true,
            no_decompress: false,
            num_records: 100000,
        };
        let result = invoke(module_name, &target_source, &options).unwrap();

        assert_eq!(result.error_message(), Some(&error_message.to_string()));
    }

    #[test]
    fn test_empty_invoke() {
        let empty_input_path = PathBuf::from("./tests/inputs/empty");

        invoke_wrapper_determine_pass(
            "empty",
            &empty_input_path,
            "plain text format (unformatted)",
            "http://edamontology.org/format_1964",
        );

        let not_empty_input_path = PathBuf::from("./tests/inputs/toy.fa");
        invoke_wrapper_determine_fail("empty", &not_empty_input_path, "The file is not empty");
    }

    #[test]
    fn test_bam_invoke() {
        let bam_input_path = PathBuf::from("./tests/inputs/toy.bam");

        invoke_wrapper_determine_pass(
            "bam",
            &bam_input_path,
            "BAM",
            "http://edamontology.org/format_2572",
        );

        let not_bam_input_path = PathBuf::from("./tests/inputs/toy.sam");
        invoke_wrapper_determine_fail("bam", &not_bam_input_path, "failed to fill whole buffer");
    }

    #[test]
    fn test_bcf_invoke() {
        let bcf_input_path = PathBuf::from("./tests/inputs/toy.bcf");

        invoke_wrapper_determine_pass(
            "bcf",
            &bcf_input_path,
            "BCF",
            "http://edamontology.org/format_3020",
        );

        let not_bcf_input_path = PathBuf::from("./tests/inputs/toy.vcf");
        invoke_wrapper_determine_fail("bcf", &not_bcf_input_path, "failed to fill whole buffer");
    }

    #[test]
    fn test_bed_invoke() {
        let bed_input_path = PathBuf::from("./tests/inputs/toy.bed");

        invoke_wrapper_determine_pass(
            "bed",
            &bed_input_path,
            "BED",
            "http://edamontology.org/format_3003",
        );

        let not_bed_input_path = PathBuf::from("./tests/inputs/toy.fa");
        invoke_wrapper_determine_fail("bed", &not_bed_input_path, "missing start position");
    }

    #[test]
    fn test_cram_invoke() {
        let cram_input_path = PathBuf::from("./tests/inputs/toy.cram");

        // TODO: review this later. currently the assert fails due to the CRAM file not being valid.
        invoke_wrapper_determine_pass(
            "cram",
            &cram_input_path,
            "CRAM",
            "http://edamontology.org/format_3462",
        );

        let not_cram_input_path = PathBuf::from("./tests/inputs/toy.bam");
        invoke_wrapper_determine_fail("cram", &not_cram_input_path, "invalid CRAM header");
    }

    #[test]
    fn test_fasta_invoke() {
        let fasta_input_path = PathBuf::from("./tests/inputs/toy.fa");

        invoke_wrapper_determine_pass(
            "fasta",
            &fasta_input_path,
            "FASTA",
            "http://edamontology.org/format_1929",
        );

        let not_fasta_input_path = PathBuf::from("./tests/inputs/toy.sam");
        invoke_wrapper_determine_fail("fasta", &not_fasta_input_path, "missing prefix ('>')");
    }

    #[test]
    fn test_fastq_invoke() {
        let fastq_input_path = PathBuf::from("./tests/inputs/toy.fq");

        invoke_wrapper_determine_pass(
            "fastq",
            &fastq_input_path,
            "FASTQ",
            "http://edamontology.org/format_1930",
        );

        let not_fastq_input_path = PathBuf::from("./tests/inputs/toy.fa");
        invoke_wrapper_determine_fail("fastq", &not_fastq_input_path, "invalid name prefix");
    }

    #[test]
    fn test_gff3_invoke() {
        let gff3_input_path = PathBuf::from("./tests/inputs/toy.gff3");

        invoke_wrapper_determine_pass(
            "gff3",
            &gff3_input_path,
            "GFF3",
            "http://edamontology.org/format_1975",
        );

        let not_gff3_input_path = PathBuf::from("./tests/inputs/toy.gtf");
        invoke_wrapper_determine_fail("gff3", &not_gff3_input_path, "invalid record");
    }

    #[test]
    fn test_gtf_invoke() {
        let gtf_input_path = PathBuf::from("./tests/inputs/toy.gtf");

        invoke_wrapper_determine_pass(
            "gtf",
            &gtf_input_path,
            "GTF",
            "http://edamontology.org/format_2306",
        );

        let not_gtf_input_path = PathBuf::from("./tests/inputs/toy.gff3");
        invoke_wrapper_determine_fail("gtf", &not_gtf_input_path, "invalid record");
    }

    #[test]
    fn test_sam_invoke() {
        let sam_input_path = PathBuf::from("./tests/inputs/toy.sam");

        invoke_wrapper_determine_pass(
            "sam",
            &sam_input_path,
            "SAM",
            "http://edamontology.org/format_2573",
        );

        let not_sam_input_path = PathBuf::from("./tests/inputs/toy.fa");
        invoke_wrapper_determine_fail("sam", &not_sam_input_path, "invalid flags");
    }

    #[test]
    fn test_vcf_invoke() {
        let vcf_input_path = PathBuf::from("./tests/inputs/toy.vcf");

        invoke_wrapper_determine_pass(
            "vcf",
            &vcf_input_path,
            "VCF",
            "http://edamontology.org/format_3016",
        );

        let not_vcf_input_path = PathBuf::from("./tests/inputs/toy.bed");
        invoke_wrapper_determine_fail("vcf", &not_vcf_input_path, "empty input");
    }
}
