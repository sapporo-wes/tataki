mod args;
mod edam;
mod ext_tools;
mod fetch;
mod logger;
mod parser;
mod run;

use anyhow::{anyhow, Result};
use clap::Parser;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::process::exit;

use crate::args::OutputFormat;
use crate::edam::EDAM_MAP;
use crate::run::Config;

fn main() -> Result<()> {
    // parse arguments and options with clap
    let args = args::Args::parse();
    logger::init_logger(args.verbose, args.quiet);
    info!("tataki started");
    debug!("args: {:?}", args);
    debug!("output format: {:?}", args.get_output_format());

    let config: Config = match &args.conf {
        None => {
            let config_str = include_str!("./tataki.conf");
            serde_yaml::from_str(config_str)?
        }
        Some(path) => {
            let config_file = File::open(path)?;
            let reader = BufReader::new(config_file);
            serde_yaml::from_reader(reader)?
        }
    };

    if args.dry_run {
        run::dry_run(config)?;
    } else {
        run::run(config, args)?;
    }

    Ok(())
}
