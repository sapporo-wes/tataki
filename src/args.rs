use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum OutputFormat {
    Yaml,
    Tsv,
    Csv,
    Json,
}

#[derive(Parser, Debug)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    version = env!("CARGO_PKG_VERSION"),
    after_help = concat!("Version: v", env!("CARGO_PKG_VERSION")),
    arg_required_else_help = true,
)]

pub struct Args {
    /// Path to the file
    #[clap(name = "FILE|URL", required_unless_present = "dry_run")]
    pub input: Vec<String>,

    /// Path to the output file [default: stdout]
    #[clap(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    #[clap(short = 'f', value_enum, default_value = "csv",conflicts_with_all = ["yaml"])]
    output_format: OutputFormat,

    #[clap(long, hide = true)]
    yaml: bool,

    /// Specify the directory in which to create a temporary directory. If this option is not provided, a temporary directory will be created in the default system temporary directory (/tmp).
    #[clap(short = 'C', long, value_name = "DIR")]
    pub cache_dir: Option<PathBuf>,

    // #[clap(long, hide = true)]
    // pub full_fetch: bool,
    /// Specify the tataki configuration file. If this option is not provided, the default configuration will be used.
    /// The option `--dry-run` shows the default configuration file.
    #[clap(short, long, value_name = "FILE")]
    pub conf: Option<PathBuf>,

    /// Attempt to read the whole lines from the input files.
    #[clap(short, long, conflicts_with_all = ["num_records"])]
    pub tidy: bool,

    /// Do not try to decompress the input file when detecting the file format.
    #[clap(long)]
    pub no_decompress: bool,

    /// Number of records to read from the input file. Recommened to set it to a multiple of 4 to prevent false negatives. Conflicts with `--tidy` option.
    #[clap(short, long, default_value = "100000", value_parser = validate_num_records_greater_than_zero)]
    pub num_records: usize,

    /// Output the configuration file in yaml format and exit the program. If `--conf` option is not provided, the default configuration file will be shown.
    #[clap(long)]
    pub dry_run: bool,

    /// Show verbose log messages
    #[clap(short, long)]
    pub verbose: bool,

    /// Suppress all log messages
    #[clap(short, long)]
    pub quiet: bool,
}

impl Args {
    pub fn get_output_format(&self) -> OutputFormat {
        if self.yaml {
            OutputFormat::Yaml
        } else {
            self.output_format
        }
    }
}

fn validate_num_records_greater_than_zero(v: &str) -> Result<usize> {
    let num = v
        .parse::<usize>()
        .with_context(|| format!("Failed to parse {}", v))?;
    anyhow::ensure!(
        num > 0,
        "The number of records to read must be greater than 0"
    );
    Ok(num)
}
