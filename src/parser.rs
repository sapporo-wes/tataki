mod bam;
mod bcf;
mod bed;
mod cram;
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
    /// Determine whether the given file is the format that this parser can parse.
    /// If the given file is the format that this parser can parse, return `Ok(ModuleResult)`.
    /// Otherwise, return `Err(anyhow::Error)` and the error message should provide the reason why this parser cannot parse the given file.
    /// To create `ModuleResult`, use `ModuleResult::with_result()` which takes `is_ok`, `label`, `id`, and `error_message` as arguments.
    /// `is_ok`: true if the given file is the format that this parser can parse, false otherwise.
    /// `label`:  
    fn determine(&self, input_path: &Path) -> Result<ModuleResult>;
}

pub fn from_str_to_parser(module_name: &str) -> Result<Box<dyn Parser>> {
    let module_name = module_name.to_lowercase();
    match &module_name[..] {
        "bam" => Ok(Box::new(bam::Bam)),
        "bcf" => Ok(Box::new(bcf::Bcf)),
        "bed" => Ok(Box::new(bed::Bed)),
        "cram" => Ok(Box::new(cram::Cram)),
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

// Result<()>ではなく、Result<ModuleResult>を返すようにしているのは、
// determine()自体の成功可否をModuleResult.is_ok、他の処理の成功可否をOk/Errで表現できるようにするため
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
