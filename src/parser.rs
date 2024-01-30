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
mod vcf;

use anyhow::{bail, Result};
use log::info;
use std::path::Path;

use crate::run::ModuleResult;

pub trait Parser {
    /// Determine if the provided file is in a format that this parser can interpret.
    /// If the parser can successfully interpret the file, return `Ok(ModuleResult)`.
    /// If it does not, return `Err(anyhow::Error)`, including an error message that specifies the reasons why the parser cannot process the file.
    /// To construct `ModuleResult`, utilize `ModuleResult::with_result()` which requires `label` and `id` as parameters.
    /// `id`: EDAM Class ID
    /// `label`: EDAM Preferred Label
    fn determine(&self, input_path: &Path) -> Result<ModuleResult>;
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
        _ => bail!("Unsupported parser name: {}", module_name),
    }
}

// Return the result of determine() using Ok(ModuleResult), and return errors in other parts using Err.
pub fn invoke(module_name: &str, target_file_path: &Path) -> Result<ModuleResult> {
    info!("Invoking parser {}", module_name);

    let parser = from_str_to_parser(module_name)?;

    Ok(parser.determine(target_file_path).unwrap_or_else(|e| {
        let mut module_result = ModuleResult::with_result(None, None);
        module_result.set_is_ok(false);
        module_result.set_error_message(e.to_string());
        module_result
    }))
}
