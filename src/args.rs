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
    after_help = concat!("Version: ", env!("CARGO_PKG_VERSION")),
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
    #[clap(long, value_name = "DIR")]
    pub cache_dir: Option<PathBuf>,

    // TODO
    // #[clap(long, hide = true)]
    // pub full_fetch: bool,
    /// Specify the tataki configuration file. If this option is not provided, the default configuration will be used.
    /// The option `--dry-run` shows the default configuration file.
    #[clap(short, long, value_name = "FILE")]
    pub conf: Option<PathBuf>,

    /// Output the configuration file in yaml format and exit the program. If `--conf` option is not provided, the default configuration file will be shown.
    #[clap(long)]
    pub dry_run: bool,

    /// Sets the level of verbosity
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
