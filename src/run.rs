use anyhow::{anyhow, bail, Result};
use log::{debug, error, info, warn};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};
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
// TODO あとでこのpub外す
pub struct ModuleResult {
    target_file_path: PathBuf,
    is_ok: bool,
    label: Option<String>,
    id: Option<String>,
    error_message: Option<String>,
    is_edam: bool,
}

// TODO こいついらなくなったかも。
impl Serialize for ModuleResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_map(Some(2))?;

        // Someの中身がNoneの場合は、serialize_entryしない。

        state.serialize_entry(
            &self.target_file_path,
            &HashMap::from([("id", &self.id), ("label", &self.label)]),
        )?;
        state.end()
    }
}

impl ModuleResult {
    // TODO こいつで書き直す
    pub fn with_result(label: Option<String>, id: Option<String>) -> Self {
        Self {
            target_file_path: PathBuf::new(),
            is_ok: true,
            label,
            id,
            error_message: None,
            is_edam: true,
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
}

// Struct to deserialize the contents of the conf file.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    order: Vec<Operation>,
}

// Enum to represent the operation and to deserialize the contents of the conf file.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Operation {
    // module name
    Default(String),

    Custom(HashMap<String, String>),
}

pub fn run(config: Config, args: Args) -> Result<()> {
    let temp_dir = fetch::create_temporary_dir(&args.cache_dir)?;
    info!("Created temporary directory: {}", temp_dir.path().display());

    let mut module_results: Vec<ModuleResult> = Vec::new();

    for input in args.input {
        // Prepare input file path from url or local file path.
        // Download the file and store it in the specified cache directory if input is url.
        // let target_file_path = match input.as_ref().and_then(|input| Url::parse(input).ok()) {
        let target_file_path = match Url::parse(&input).ok() {
            Some(url) => crate::fetch::download_from_url(url, &temp_dir)?,
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

    let mut serialized_map = HashMap::new();
    for module_result in &module_results {
        let target_file_path = &module_result.target_file_path;
        serialized_map.insert(
            target_file_path.clone(),
            HashMap::from([("id", &module_result.id), ("label", &module_result.label)]),
        );
    }

    println!("----");
    let yaml_str = serde_yaml::to_string(&serialized_map)?;
    println!("test_output1:\n\n{}", yaml_str);

    // TODO この出力方法だと、yamlが配列になっちゃう。消す？
    let module_results_str = serde_yaml::to_string(&module_results)?;
    println!("test_output2:\n\n{}", module_results_str);

    Ok(())
}

fn run_modules(
    target_file_path: PathBuf,
    config: &Config,
    // cache_dir: &Option<PathBuf>,
    temp_dir: &TempDir,
) -> Result<ModuleResult> {
    let cwl_input_file_path: Option<NamedTempFile> = if cwl_module_exists(config)? {
        Some(ext_tools::make_cwl_input_file(
            target_file_path.clone(),
            temp_dir,
        )?)
    } else {
        None
    };

    for item in &config.order {
        let (operation_name, module) = match item {
            Operation::Default(module) => (None, module),
            Operation::Custom(custom) => {
                let (operation_name, module) = custom
                    .iter()
                    .next()
                    .ok_or_else(|| anyhow!("Invalid custom operation specified."))?;
                (Some(operation_name), module)
            }
        };

        let module_path = Path::new(&module);
        let module_extension = module_path
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("");

        let mut module_result = match module_extension {
            "" => parser::invoke(module, &target_file_path)?,
            "cwl" => ext_tools::invoke(
                module_path,
                &target_file_path,
                cwl_input_file_path.as_ref().unwrap(),
            )?,
            _ => anyhow::bail!(
                "An unsupported file extension was specified for the module value in the conf file"
            ),
        };

        module_result.set_target_file_path(target_file_path.clone());

        if module_result.is_ok {
            info!("Detected!! {}", module);

            if let Some(operation_name) = operation_name {
                module_result.label = Some(operation_name.clone());
                module_result.id = None;
                module_result.is_edam = false;
            }

            // TODO : for debug. delete later
            // println!("\nend {:?}", &module_result);
            return Ok(module_result);
        } else {
            debug!(
                "Module \"{}\" failed. Reason:\n{}",
                module,
                module_result.error_message.unwrap_or("".to_string())
            );
        }
    }

    // Found that no module can handle the input file, so return ModuleResult with is_ok=false.
    return Ok(ModuleResult {
        target_file_path,
        is_ok: false,
        label: None,
        id: None,
        error_message: None,
        is_edam: false,
    });
}

pub fn dry_run(config: Config) -> Result<()> {
    // output the configuration file in yaml format
    let yaml = serde_yaml::to_string(&config)?;
    println!("{}", yaml);

    Ok(())
}

fn cwl_module_exists(config: &Config) -> Result<bool> {
    for item in &config.order {
        // TODO これ重複する作業なので、Operationにimplしてまとめたい
        let (_, module) = match item {
            Operation::Default(module) => (None, module),
            Operation::Custom(custom) => {
                let (operation_name, module) = custom
                    .iter()
                    .next()
                    .ok_or_else(|| anyhow!("Invalid custom operation specified."))?;
                (Some(operation_name), module)
            }
        };

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
