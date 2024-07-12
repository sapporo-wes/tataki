use anyhow::{Context, Result};
use clap::Parser;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<()> {
    // parse arguments and options with clap
    let args = tataki::args::Args::parse();

    let config: tataki::module::Config = match &args.conf {
        None => {
            let config_str = include_str!("./tataki.conf");
            serde_yaml::from_str(config_str)?
        }
        Some(path) => {
            let config_file = File::open(path).with_context(|| {
                format!("Failed to open the config file: {}", path.to_str().unwrap(),)
            })?;
            let reader = BufReader::new(config_file);
            serde_yaml::from_reader(reader).with_context(|| {
                format!(
                    "Failed to parse the config file: {}",
                    path.to_str().unwrap(),
                )
            })?
        }
    };

    // exit the program with exit code 0
    // exit(0);

    if args.dry_run {
        tataki::module::dry_run(config)?;
    } else {
        tataki::module::run(config, args)?;
    }

    Ok(())
}
