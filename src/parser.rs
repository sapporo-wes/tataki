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
    /// Determine if the provided file is in a format that this parser can interpret.
    /// If the parser can successfully interpret the file, return `Ok(ModuleResult)`.
    /// If it does not, return `Err(anyhow::Error)`, including an error message that specifies the reasons why the parser cannot process the file.
    /// To construct `ModuleResult`, utilize `ModuleResult::with_result()` which requires `label` and `id` as parameters.
    /// `id`: EDAM Class ID
    /// `label`: EDAM Preferred Label
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

    // Test the result of invoking the parser when determine is successful.
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

    // Test the result of invoking the parser when determine is unsuccessful.
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
}
