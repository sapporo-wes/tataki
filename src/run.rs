use anyhow::{anyhow, bail, Context, Result};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, TempDir};
use url::Url;

use crate::args::Args;
use crate::ext_tools;
use crate::fetch;
use crate::parser;
use crate::OutputFormat;

// Struct to store the result of Parser invocation and ExtTools invocation.
#[derive(Debug)]
pub struct ModuleResult {
    target_file_path: PathBuf,
    is_ok: bool,
    label: Option<String>,
    id: Option<String>,
    error_message: Option<String>,
}

impl ModuleResult {
    pub fn with_result(label: Option<String>, id: Option<String>) -> Self {
        Self {
            target_file_path: PathBuf::new(),
            is_ok: true,
            label,
            id,
            error_message: None,
        }
    }

    pub fn set_is_ok(&mut self, is_ok: bool) {
        self.is_ok = is_ok;
    }

    pub fn set_error_message(&mut self, error_message: String) {
        self.error_message = Some(error_message);
    }

    pub fn set_target_file_path(&mut self, target_file_path: PathBuf) {
        self.target_file_path = target_file_path;
    }

    pub fn create_module_results_string(
        module_results: &[ModuleResult],
        format: OutputFormat,
    ) -> Result<String> {
        fn csv_serialize(module_results: &[ModuleResult], delimiter: u8) -> Result<String> {
            let mut data = Vec::new();
            {
                let mut writer = csv::WriterBuilder::new()
                    .delimiter(delimiter)
                    .from_writer(&mut data);

                writer.write_record(["File Path", "Edam ID", "Label"])?;

                for module_result in module_results.iter() {
                    let target_file_path = &module_result.target_file_path;
                    writer.serialize((
                        target_file_path.to_str().with_context(|| {
                            format!(
                                "Failed to convert the file path to a string: {}",
                                target_file_path.display()
                            )
                        })?,
                        &module_result.id,
                        &module_result.label,
                    ))?;
                }
            }

            let data_str = String::from_utf8_lossy(&data);
            Ok(data_str.into_owned())
        }

        match format {
            OutputFormat::Yaml => {
                let mut serialized_map = HashMap::new();
                for module_result in module_results {
                    let target_file_path = &module_result.target_file_path;
                    serialized_map.insert(
                        target_file_path.clone(),
                        HashMap::from([("id", &module_result.id), ("label", &module_result.label)]),
                    );
                }

                let yaml_str = serde_yaml::to_string(&serialized_map)?;
                Ok(yaml_str)
            }
            OutputFormat::Tsv => csv_serialize(module_results, b'\t'),
            OutputFormat::Csv => csv_serialize(module_results, b','),
            OutputFormat::Json => {
                let mut serialized_map = HashMap::new();
                for module_result in module_results {
                    let target_file_path = &module_result.target_file_path;
                    serialized_map.insert(
                        target_file_path.clone(),
                        HashMap::from([("id", &module_result.id), ("label", &module_result.label)]),
                    );
                }

                let json_str = serde_json::to_string(&serialized_map)?;
                Ok(json_str)
            }
        }
    }
}

// Struct to deserialize the contents of the conf file.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    order: Vec<String>,
}

pub fn run(config: Config, args: Args) -> Result<()> {
    let temp_dir = fetch::create_temporary_dir(&args.cache_dir)?;
    info!("Created temporary directory: {}", temp_dir.path().display());

    let mut module_results: Vec<ModuleResult> = Vec::new();

    // insert "empty" module at the beginning of the module order, so that the empty module is always invoked first.
    let mut config = config;
    config.order.insert(0, "empty".to_string());

    for input in &args.input {
        info!("Processing input: {}", input);

        // Prepare input file path from url or local file path.
        // Download the file and store it in the specified cache directory if input is url.
        // let target_file_path = match input.as_ref().and_then(|input| Url::parse(input).ok()) {
        let target_file_path = match Url::parse(input).ok() {
            Some(url) => {
                info!("Downloading from {}", url);
                let path = crate::fetch::download_from_url(&url, &temp_dir)?;
                info!("Downloaded to {}", path.display());
                path
            }
            None => {
                let path = PathBuf::from(input);
                if !path.exists() {
                    bail!(
                        "The specified target file does not exist. Please check the path. : {}",
                        path.display()
                    );
                }
                path
            }
        };

        //
        // let module_result = run_modules(target_file_path, &config, &args.cache_dir)?;
        let module_result = run_modules(target_file_path, &config, &temp_dir)?;
        module_results.push(module_result);
    }

    // if args.cache_dir is Some, keep the temporary directory.
    // Otherwise, delete the temporary directory.
    if args.cache_dir.is_some() {
        info!(
            "Keeping temporary directory: {}",
            temp_dir.into_path().display()
        );
    } else {
        info!(
            "Deleting temporary directory: {}",
            temp_dir.path().display()
        );
        temp_dir.close()?;
    }

    let result_str =
        ModuleResult::create_module_results_string(&module_results, args.get_output_format())?;

    // if args.output is Some, write the result to the specified file. Otherwise, write the result to stdout.
    if let Some(output_path) = args.output {
        info!("Writing the result to {}", output_path.display());
        std::fs::write(output_path, result_str)?;
    } else {
        println!("{}", result_str);
    }

    Ok(())
}

fn run_modules(
    target_file_path: PathBuf,
    config: &Config,
    temp_dir: &TempDir,
) -> Result<ModuleResult> {
    // create an input file for CWL modules if there is any CWL module in the config file.
    let cwl_input_file_path: Option<NamedTempFile> = if cwl_module_exists(config)? {
        Some(ext_tools::make_cwl_input_file(
            target_file_path.clone(),
            temp_dir,
        )?)
    } else {
        None
    };

    let module_result = config
        .order
        .iter()
        .find_map(|module| {
            let module_path = Path::new(&module);
            let module_extension = module_path
                .extension()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or("");

            let result = match module_extension {
                "" => parser::invoke(module, &target_file_path),
                "cwl" => ext_tools::invoke(
                    module_path,
                    &target_file_path,
                    cwl_input_file_path.as_ref().unwrap(),
                ),
                _ => Err(anyhow!(
                    "An unsupported file extension '.{}' was specified for the module value in the conf file. Only .cwl is supported for external extension mode.",
                    module_extension
                )),
            };

            match result {
                Ok(mut module_result) => {
                    if module_result.is_ok {
                        info!("Detected!! {}", module);
                        module_result.set_target_file_path(target_file_path.clone());
                        Some(module_result)
                    } else {
                        debug!(
                            "Module \"{}\" failed. Reason:\n{}",
                            module,
                            module_result.error_message.unwrap_or("".to_string())
                        );
                        None
                    }
                },
                Err(e) => {
                    error!("An error occurred while trying to invoke the \'{}\' module. Reason:\n{}", module, e);
                    None
                },
            }
        })
        .unwrap_or_else(|| {
            let mut none_result = ModuleResult::with_result(None, None);
            none_result.set_target_file_path(target_file_path.clone());
            none_result
        });

    Ok(module_result)
}

pub fn dry_run(config: Config) -> Result<()> {
    // output the configuration file in yaml format
    let yaml = serde_yaml::to_string(&config)?;
    println!("{}", yaml);

    Ok(())
}

fn cwl_module_exists(config: &Config) -> Result<bool> {
    for module in &config.order {
        let module_path = Path::new(&module);
        let module_extension = module_path
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("");

        if module_extension == "cwl" {
            return Ok(true);
        }
    }
    Ok(false)
}
