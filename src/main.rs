mod bar;
// mod foo;

#[allow(unused_imports)]
use anyhow::{anyhow, bail, ensure, Context, Result};
use clap::{Parser, Subcommand};
use noodles::bam;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Read, Seek};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    version = env!("CARGO_PKG_VERSION"),
    after_help = "",
    arg_required_else_help = true,
)]
pub struct Cli {
    #[clap(subcommand)]
    subcommand: Subcommands,
}

#[derive(Subcommand, Debug)]
enum Subcommands {
    #[clap(arg_required_else_help = true)]
    Foo {
        /// Sets the level of verbosity
        #[clap(short, long)]
        verbose: bool,

        /// Path to the file
        #[clap(name = "FILE")]
        input: PathBuf,
    },
    #[clap(arg_required_else_help = true)]
    Bar {
        /// Sets the level of verbosity
        #[clap(short, long)]
        verbose: bool,
    },
}
fn main() -> Result<()> {
    // parse arguments and options with clap
    let cli = Cli::parse();

    match cli.subcommand {
        Subcommands::Foo { verbose, input } => {}
        Subcommands::Bar { .. } => {}
    }

    Ok(())
}
