use anyhow::{anyhow, bail, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, TempDir};
use url::Url;

use crate::args::{Args, OutputFormat};
use crate::source::{Source, CompressedFormat};

// Struct to store the result of Parser invocation and ExtTools invocation.
#[derive(Debug)]
pub struct ModuleResult {
    input: String,
    is_ok: bool,
    label: Option<String>,
    id: Option<String>,
    error_message: Option<String>,
    decompressed: Option<DecompressedFormat>,
}

impl From<&CompressedFormat> for ModuleResult {
    fn from(compressed_format: &CompressedFormat) -> Self {
        match compressed_format {
            CompressedFormat::Bgzf => ModuleResult::with_result(None, None),
            CompressedFormat::GZ => ModuleResult::with_result(
                Some("GZIP format".to_string()),
                Some("http://edamontology.org/format_3989".to_string()),
            ),
            CompressedFormat::BZ2 => ModuleResult::with_result(None, None),
            CompressedFormat::None => ModuleResult::with_result(None, None),
        }
    }
}

impl ModuleResult {
    pub fn with_result(label: Option<String>, id: Option<String>) -> Self {
        Self {
            input: String::new(),
            is_ok: true,
            label,
            id,
            error_message: None,
            decompressed: None,
        }
    }

    pub fn label(&self) -> Option<&String> {
        self.label.as_ref()
    }

    pub fn id(&self) -> Option<&String> {
        self.id.as_ref()
    }

    pub fn error_message(&self) -> Option<&String> {
        self.error_message.as_ref()
    }

    pub fn set_is_ok(&mut self, is_ok: bool) {
        self.is_ok = is_ok;
    }

    pub fn set_error_message(&mut self, error_message: String) {
        self.error_message = Some(error_message);
    }

    pub fn set_input(&mut self, input: String) {
        self.input = input;
    }

    fn swap_edam_of_module_result_and_compressed_format(&mut self, compressed_format_edam: Self) {
        let tmp_label = self.label.to_owned();
        let tmp_id = self.id.to_owned();

        self.label = compressed_format_edam.label;
        self.id = compressed_format_edam.id;

    let tmp_decompressed = DecompressedFormat {
        label: tmp_label,
        id: tmp_id,
    };
        self.decompressed = Some(tmp_decompressed);
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

                writer.write_record(["File Path", "Edam ID", "Label", "Decompressed ID", "Decompressed Label"])?;

                for module_result in module_results.iter() {
                    writer.serialize((
                        &module_result.input,
                        &module_result.id,
                        &module_result.label,
                        &module_result.decompressed.as_ref().and_then(|d| d.id.as_ref()),
                        &module_result.decompressed.as_ref().and_then(|d| d.label.as_ref()),
                    ))?;
                }
            }

            let data_str = String::from_utf8_lossy(&data);
            Ok(data_str.into_owned())
        }

        match format {
            OutputFormat::Yaml => {
                let mut serialized_map: HashMap<String, serde_yaml::Value> = HashMap::new();
                for module_result in module_results {
                    let target_file_path = &module_result.input;

                    // create yaml map for decompressed field
                    let mut de_map : HashMap<String, serde_yaml::Value> = HashMap::new();
                    match &module_result.decompressed {
                        Some(decompressed) => {
                            match &decompressed.id {
                                Some(id) => {
                                    de_map.insert("id".to_string(), serde_yaml::Value::String(id.clone()));
                                },
                                None => {
                                    de_map.insert("id".to_string(), serde_yaml::Value::Null);
                                }
                            }
                            match &decompressed.label {
                                Some(label) => {
                                    de_map.insert("label".to_string(), serde_yaml::Value::String(label.clone()));
                                },
                                None => {
                                    de_map.insert("label".to_string(), serde_yaml::Value::Null);
                                }
                            }
                        },
                        None => {
                            de_map.insert("id".to_string(), serde_yaml::Value::Null);
                            de_map.insert("label".to_string(), serde_yaml::Value::Null);
                        }
                    }

                    // create yaml map for label and id fields
                    let mut comp_map : HashMap<String, serde_yaml::Value> = HashMap::new();
                    match &module_result.id {
                        Some(id) => {
                            comp_map.insert("id".to_string(), serde_yaml::Value::String(id.clone()));
                        },
                        None => {
                            comp_map.insert("id".to_string(), serde_yaml::Value::Null);
                        }
                    }
                    match &module_result.label {
                        Some(label) => {
                            comp_map.insert("label".to_string(), serde_yaml::Value::String(label.clone()));
                        },
                        None => {
                            comp_map.insert("label".to_string(), serde_yaml::Value::Null);
                        }
                    }

                    // add decompressed field to the yaml map
                    comp_map.insert("decompressed".to_string(), serde_yaml::to_value(de_map)?);
                    // match &module_result.decompressed {
                    //     Some(decompressed) => {
                    //         comp_map.insert("decompressed".to_string(), serde_yaml::to_value(decompressed)?);
                    //     },
                    //     None => {
                    //         comp_map.insert("decompressed".to_string(), serde_yaml::Value::Null);
                    //     }
                    // }


                    serialized_map.insert(
                        target_file_path.clone(),
                        serde_yaml::to_value(comp_map)?,
                    );
                }

                let yaml_str = serde_yaml::to_string(&serialized_map)?;
                Ok(yaml_str)
            }
            OutputFormat::Tsv => csv_serialize(module_results, b'\t'),
            OutputFormat::Csv => csv_serialize(module_results, b','),
            OutputFormat::Json => {
                let mut serialized_map: HashMap<String, serde_json::Value> = HashMap::new();
                for module_result in module_results {
                    let target_file_path = &module_result.input;

                    // create json map for decompressed field
                    let mut de_map : HashMap<String, serde_json::Value> = HashMap::new();
                    match &module_result.decompressed {
                        Some(decompressed) => {
                            match &decompressed.id {
                                Some(id) => {
                                    de_map.insert("id".to_string(), serde_json::Value::String(id.clone()));
                                },
                                None => {
                                    de_map.insert("id".to_string(), serde_json::Value::Null);
                                }
                            }
                            match &decompressed.label {
                                Some(label) => {
                                    de_map.insert("label".to_string(), serde_json::Value::String(label.clone()));
                                },
                                None => {
                                    de_map.insert("label".to_string(), serde_json::Value::Null);
                                }
                            }
                        },
                        None => {
                            de_map.insert("id".to_string(), serde_json::Value::Null);
                            de_map.insert("label".to_string(), serde_json::Value::Null);
                        }
                    }

                    // create json map for label and id fields
                    let mut comp_map : HashMap<String, serde_json::Value> = HashMap::new();
                    match &module_result.id {
                        Some(id) => {
                            comp_map.insert("id".to_string(), serde_json::Value::String(id.clone()));
                        },
                        None => {
                            comp_map.insert("id".to_string(), serde_json::Value::Null);
                        }
                    }
                    match &module_result.label {
                        Some(label) => {
                            comp_map.insert("label".to_string(), serde_json::Value::String(label.clone()));
                        },
                        None => {
                            comp_map.insert("label".to_string(), serde_json::Value::Null);
                        }
                    }

                    // add components field to the json map
                    comp_map.insert("decompressed".to_string(), serde_json::to_value(de_map)?);
                    // match &module_result.decompressed {
                    //     Some(decompressed) => {
                    //         comp_map.insert("decompressed".to_string(), serde_json::to_value(decompressed)?);
                    //     },
                    //     None => {
                    //         comp_map.insert("decompressed".to_string(), serde_json::Value::Null);
                    //     }
                    // }




                    serialized_map.insert(
                        target_file_path.clone(),
                        serde_json::to_value(comp_map)?,
                    );
                }

                let json_str = serde_json::to_string(&serialized_map)?;
                Ok(json_str)
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DecompressedFormat {
    label: Option<String>,
    id: Option<String>,
}

// Struct to deserialize the contents of the conf file.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    order: Vec<String>,
}

pub struct InvokeOptions {
    pub tidy: bool,
    pub no_decompress: bool,
    pub num_records: usize,
}

impl From<&Args> for InvokeOptions {
    fn from(args: &Args) -> Self {
        Self {
            tidy: args.tidy,
            no_decompress: args.no_decompress,
            num_records: args.num_records,
        }
    }
}



pub fn run(config: Config, args: Args) -> Result<()> {
    crate::logger::init_logger(args.verbose, args.quiet);
    info!("tataki started");
    debug!("Args: {:?}", args);
    debug!("Output format: {:?}", args.get_output_format());

    let invoke_options = InvokeOptions::from(&args);

    let cwl_module_exists = cwl_module_exists(&config)?;

    // validate the user-provided options and input arguments to ensure they are suitable for execution.
    check_run_condition_cwl_module(&args.input, cwl_module_exists, &invoke_options)?;

    let temp_dir = crate::fetch::create_temporary_dir(&args.cache_dir)?;
    info!("Created temporary directory: {}", temp_dir.path().display());

    // create an empty vector to store the results of each module invocation.
    let mut module_results: Vec<ModuleResult> = Vec::new();

    // insert "empty" module at the beginning of the module order, so that the empty module is always invoked first.
    let mut config = config;
    config.order.insert(0, "empty".to_string());

    for input in &args.input {
        let mut input = input.clone();
        info!("Processing input: {}", input);

        // Check if the input is stdin or path. If path, download the file if it is a url.
        let (target_source , compressed_format) = match input.parse::<Source>()? {
            Source::FilePath(p) => {
                // Prepare input file path from url or local file path.
                // Download the file and store it in the specified cache directory if input is url.
                let target_file_path = match Url::parse(&p.to_string_lossy()).ok() {
                    Some(url) => {
                        info!("Downloading from {}", url);
                        let path = crate::fetch::download_from_url(&url, &temp_dir)?;
                        info!("Downloaded to {}", path.display());
                        path
                    }
                    None => {
                        let path = PathBuf::from(&input);
                        if !path.exists() {
                            bail!("The specified target file does not exist. Please check the path. : {}" ,path.display()
                            );
                        }
                        path
                    }
                };

                let (source, compressed_format) = Source::decompress_into_tempfile_from_filepath_if_needed(
                    &target_file_path,
                    &invoke_options,
                    &temp_dir,
                    cwl_module_exists,
                )?;

                match source {
                    Some(source) => {(source, compressed_format)},
                    None =>          {  (    Source::FilePath(target_file_path), compressed_format)},
                }
            }
            Source::Stdin => {
                info!("Reading from STDIN...");
                input = "STDIN".to_string();
                Source::convert_into_tempfile_from_stdin(&invoke_options, &temp_dir)?
            },
            Source::TempFile(_) => unreachable!(),
            Source::Memory(_) => unreachable!(),
        };

        let mut module_result = run_modules(target_source, &config, &temp_dir, &invoke_options)?;

        let compressed_format_edam = ModuleResult::from(&compressed_format);
        // must swap the edam of the module result and the compressed format if decompress has been done.
        match compressed_format {
            CompressedFormat::None =>{},
            CompressedFormat::Bgzf => {},
            _ => {
                module_result.swap_edam_of_module_result_and_compressed_format(compressed_format_edam);
            }
        }

        module_result.set_input(input.clone());
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
    // target_file_path: PathBuf,
    target_source: Source,
    config: &Config,
    temp_dir: &TempDir,
    invoke_options: &InvokeOptions,
) -> Result<ModuleResult> {
    // Create an input file for CWL modules if there is any CWL module in the config file and input is not stdin.
    let cwl_input_file_path: Option<NamedTempFile> =
        // Check whether the input is not stdin
        if let Source::FilePath(target_file_path) = &target_source {
            // Check whether CWL modules exist in the config file.
            if cwl_module_exists(config)? {
                Some(crate::ext_tools::make_cwl_input_file(
                    target_file_path.clone(),
                    temp_dir,
                )?)
            } else {
                None
            }
        } 
        // Not invoking CWL module if the input is stdin, therefore return None here.
        else {
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
                "" => crate::parser::invoke(module, &target_source, invoke_options),
                "cwl" => {
                    // TODO might want to refactor this without using match statement.
                    // CWL module invocation is skipped if the input is not a file path or URL.
                    match target_source.as_path() {
                        Some(target_file_path) => {
                            crate::ext_tools::invoke(
                            module_path,
                            target_file_path,
                            cwl_input_file_path.as_ref().unwrap(),
                            invoke_options,)
                        },
                        None =>  Err(anyhow!("Skipping CWL module invocation. CWL modules can only be invoked with file paths or URLs.")),
                    }

                },
                _ => Err(anyhow!(
                    "An unsupported file extension '.{}' was specified for the module value in the conf file. Only .cwl is supported for external extension mode.",
                    module_extension
                )),
            };

            match result {
                Ok(module_result) => {
                    if module_result.is_ok {
                        info!("Detected!! {}", module);
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
                    // TODO fix an issue that a error here is absorbed by the find_map function.
                    warn!("An error occurred while trying to invoke the \'{}\' module. Reason:\n{}", module, e);
                    None
                },
            }
        })
        .unwrap_or_else(|| {ModuleResult::with_result(None, None)});

    // std::thread::sleep(std::time::Duration::from_secs(50));

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

fn check_run_condition_cwl_module(inputs: &[String], cwl_module_exists: bool, invoke_options: &InvokeOptions) -> Result<()> {
    // whether stdin is included in the input
    let stdin_exists = inputs.iter().any(|input| input == "-");

    /*
    cwl 
    - filepath
        - plain, --no-decompress
            - ok    
        - compressed
            - tidy needed
    - stdin
        - tidy needed
     */

    if cwl_module_exists && stdin_exists && !invoke_options.tidy {
        bail!("The `--tidy` option is required when reading from STDIN and invoking CWL modules.");
    }

    Ok(())
}
