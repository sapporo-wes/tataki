mod args;
mod edam;
mod ext_tools;
mod fetch;
mod logger;
mod parser;
mod run;

use anyhow::{Context, Result};
use clap::Parser;
use log::{debug, info};
use std::fs::File;
use std::io::BufReader;

use crate::args::OutputFormat;
use crate::run::Config;

fn main() -> Result<()> {
    // parse arguments and options with clap
    let args = args::Args::parse();
    logger::init_logger(args.verbose, args.quiet);

    let config: Config = match &args.conf {
        None => {
            let config_str = include_str!("./tataki.conf");
            serde_yaml::from_str(config_str)?
        }
        Some(path) => {
            let config_file = File::open(path)?;
            let reader = BufReader::new(config_file);
            serde_yaml::from_reader(reader).with_context(|| {
                format!(
                    "Failed to parse the config file: {}",
                    path.to_str().unwrap(),
                )
            })?
        }
    };

    if args.dry_run {
        run::dry_run(config)?;
    } else {
        info!("tataki started");
        debug!("Args: {:?}", args);
        debug!("Output format: {:?}", args.get_output_format());
        run::run(config, args)?;
    }

    Ok(())
}
