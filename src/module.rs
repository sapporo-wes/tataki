use anyhow::{anyhow, bail, Result};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, TempDir};
use url::Url;

use crate::args::{Args, OutputFormat};
use crate::source::Source;
use crate::buffered_read_seek::OnetimeRewindableReader;

// Struct to store the result of Parser invocation and ExtTools invocation.
#[derive(Debug)]
pub struct ModuleResult {
    input: String,
    is_ok: bool,
    label: Option<String>,
    id: Option<String>,
    error_message: Option<String>,
}

impl ModuleResult {
    pub fn with_result(label: Option<String>, id: Option<String>) -> Self {
        Self {
            input: String::new(),
            is_ok: true,
            label,
            id,
            error_message: None,
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
                    writer.serialize((
                        &module_result.input,
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
                    let target_file_path = &module_result.input;
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
                    let target_file_path = &module_result.input;
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

pub struct InvokeOptions {
    pub tidy: bool,
    pub no_decompress: bool,
    pub num_lines: usize,
}

impl From<&Args> for InvokeOptions {
    fn from(args: &Args) -> Self {
        Self {
            tidy: args.tidy,
            no_decompress: args.no_decompress,
            num_lines: args.num_lines,
        }
    }
}

pub fn run(config: Config, args: Args) -> Result<()> {
    crate::logger::init_logger(args.verbose, args.quiet);
    info!("tataki started");
    debug!("Args: {:?}", args);
    debug!("Output format: {:?}", args.get_output_format());

    // let invoke_options = InvokeOptions::from(&args);
    let invoke_options = InvokeOptions::from(&args);

    let temp_dir = crate::fetch::create_temporary_dir(&args.cache_dir)?;
    info!("Created temporary directory: {}", temp_dir.path().display());

    let mut module_results: Vec<ModuleResult> = Vec::new();

    // insert "empty" module at the beginning of the module order, so that the empty module is always invoked first.
    let mut config = config;
    config.order.insert(0, "empty".to_string());

    for input in &args.input {
        info!("Processing input: {}", input);

        // TODO: ここで、inputがstdinだったらtidyゆるさないよ、とかやる必要がある。
        // TODO: --tidy + 圧縮path の場合はreaderで渡したほうがいい気がする。 
        // Check if the input is stdin or path. If path, download the file if it is a url.
        let target_source = match input.parse::<Source>()? {
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
                        let path = PathBuf::from(input);
                        if !path.exists() {
                            bail!("The specified target file does not exist. Please check the path. : {}" ,path.display()
                            );
                        }
                        path
                    }
                };
                Source::FilePath(target_file_path)
                // TODO 必要だったらtempfileにせなかん
            }
            Source::Stdin => {
                Source::convert_into_tempfile_from_stdin(&invoke_options, &temp_dir)?
            },
            Source::TempFile(_) => unreachable!(),
            Source::Memory(_) => unreachable!(),
        };

        let mut module_result = run_modules(target_source, &config, &temp_dir, &invoke_options)?;


        module_result.set_input(input.clone());
        module_results.push(module_result);

        // TODO ここでSource::TempFileを使った場合は消す。

        
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
    mut target_source: Source,
    config: &Config,
    temp_dir: &TempDir,
    invoke_options: &InvokeOptions,
) -> Result<ModuleResult> {
    // Create an input file for CWL modules if there is any CWL module in the config file and input is not stdin.
    let cwl_input_file_path: Option<NamedTempFile> =
        // Check whether the input is not stdin
        if let Source::FilePath(target_file_path) = &target_source {
            // Check whether CWL modules exist in teh config file.
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
                "" => crate::parser::invoke(module, &mut target_source, invoke_options),
                "cwl" => {
                    // let target_file_path = target_source.as_path().
                    // TODO ここ、match使わずに書けないか？
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
                    error!("An error occurred while trying to invoke the \'{}\' module. Reason:\n{}", module, e);
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
