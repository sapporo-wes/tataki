use anyhow::{anyhow, bail, Context, Result};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::{Builder, NamedTempFile, TempDir};

use crate::edam;
use crate::module::{InvokeOptions, ModuleResult};

const CWL_INSPECTOR_DOCKER_IMAGE: &str = "ghcr.io/tom-tan/cwl-inspector:v0.1.1";
const LABEL_KEY: &str = "label";
const EDAM_ID_KEY: &str = "edam_id";

pub fn invoke(
    cwl_file_path: &Path,
    target_file_path: &Path,
    cwl_input_file_path: &NamedTempFile,
    _options: &InvokeOptions,
) -> Result<ModuleResult> {
    info!("Invoking ext_tools {}", cwl_file_path.display());

    let docker_path = docker_path()?;
    debug!(
        "The path of the docker command in your environment: {:?}.",
        docker_path
    );

    // make sure that the both paths are canonicalized.
    let target_file_path = target_file_path.canonicalize().with_context(|| {
        format!(
            "Something went wrong. Target file '{}' does not exist. Please try again.",
            target_file_path.display()
        )
    })?;
    let cwl_file_path = cwl_file_path
        .canonicalize()
        .with_context(|| format!("The specified path of CWL document '{}' does not exist. Please check the path for typos and try again.", cwl_file_path.display()))?;

    // get the EDAM_ID and LABEL from the comment lines in the CWL file.
    let mut cwl_metadatas = get_metadata_fields_from_cwl_file(&cwl_file_path)?;
    validate_id_and_label(&mut cwl_metadatas, &cwl_file_path)?;

    // create a docker commandline from the CWL file using cwl-inspector.
    let inspector_process = std::process::Command::new("docker")
        .args([
            "run",
            "--rm",
            "-i",
            "-v",
            &format!("{}:/usr/bin/docker:ro", docker_path.to_str().unwrap()),
            "-v",
            &format!(
                "{}:/workdir/input_file.yaml:ro",
                cwl_input_file_path.path().to_str().unwrap()
            ),
            "-v",
            &format!("{}:/workdir/module.cwl", cwl_file_path.to_str().unwrap()),
            "--workdir=/workdir",
            CWL_INSPECTOR_DOCKER_IMAGE,
            "./module.cwl",
            "commandline",
            "-i",
            "./input_file.yaml",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()?;

    let tmp_cwl_docker_command = if inspector_process.status.success() {
        String::from_utf8(inspector_process.stdout)?
    } else {
        let stderr = String::from_utf8_lossy(&inspector_process.stderr);
        bail!("Failed to run 'cwl-inspector' image:\n{}", stderr);
    };

    // split the docker command into a vector of strings.
    let tmp_cwl_docker_command_split = shlex::split(&tmp_cwl_docker_command)
        .ok_or_else(|| anyhow!("Failed to create a docker command from the CWL file."))?;

    // remove the "-v" options and split the docker command on the "-v" option.
    let mut parts_iter = tmp_cwl_docker_command_split.into_iter().peekable();
    let cwl_docker_command_name = parts_iter.next().unwrap();
    let mut cwl_docker_args_before_v: Vec<String> = Vec::new();
    let mut cwl_docker_args_after_v: Vec<String> = Vec::new();
    let mut encountered_v = false;
    while let Some(part) = parts_iter.next() {
        if part == "-v" {
            encountered_v = true;
            parts_iter.next();
        } else if encountered_v {
            cwl_docker_args_after_v.push(part);
        } else {
            cwl_docker_args_before_v.push(part);
        }
    }

    // add mount option for the input file to the docker command.
    let target_file_name = target_file_path.file_name().ok_or_else(|| {
        anyhow!(
            "Failed to get the file name from {}",
            target_file_path.display()
        )
    })?;
    let input_mount_str = format!(
        "{}:/var/lib/cwl/inputs/{}:ro",
        target_file_path.to_str().unwrap(),
        target_file_name.to_str().unwrap()
    );
    cwl_docker_args_before_v.push("-v".to_string());
    cwl_docker_args_before_v.push(input_mount_str);

    // run the docker command created by cwl-inspector.
    debug!(
        "Running the docker command: '{} {} {}'",
        cwl_docker_command_name,
        cwl_docker_args_before_v.join(" "),
        cwl_docker_args_after_v.join(" ")
    );

    let cwl_docker_process = std::process::Command::new(cwl_docker_command_name)
        .args(cwl_docker_args_before_v)
        .args(cwl_docker_args_after_v)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()?;

    let mut module_result = ModuleResult::with_result(
        cwl_metadatas.get(LABEL_KEY).map(|s| s.to_string()),
        cwl_metadatas.get(EDAM_ID_KEY).map(|s| s.to_string()),
    );

    module_result.set_is_ok(cwl_docker_process.status.success());
    if !cwl_docker_process.status.success() {
        let stderr = String::from_utf8_lossy(&cwl_docker_process.stderr);
        module_result.set_error_message(stderr.into_owned());
    };

    Ok(module_result)
}

fn docker_path() -> Result<PathBuf> {
    let process = std::process::Command::new("which")
        .arg("docker")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()?;

    if process.status.success() {
        let path = String::from_utf8(process.stdout)?;
        Ok(PathBuf::from(path.trim()))
    } else {
        bail!("Please make sure that the docker command is present in your PATH");
    }
}

#[derive(Deserialize, Debug)]
struct CwlMetadata {
    #[serde(flatten)]
    entries: HashMap<String, serde_yaml::Value>,
    #[serde(rename = "$namespaces")]
    namespaces: HashMap<String, String>,
}

fn get_metadata_fields_from_cwl_file(cwl_file_path: &Path) -> Result<HashMap<String, String>> {
    // Extract the EDAM_ID and LABEL from metadata in the CWL file. ex:
    // $namespaces:
    //   tataki: https://github.com/sapporo-wes/tataki
    // tataki:edam_id: http://edamontology.org/format_2573
    // tataki:label: SAM
    let file = std::fs::File::open(cwl_file_path)?;
    let reader = std::io::BufReader::new(file);
    let cwl_metadata: CwlMetadata = serde_yaml::from_reader(reader)
        .with_context(|| format!("Failed to parse the CWL file: {}", cwl_file_path.display()))?;

    let mut extracted_fields: HashMap<String, String> = HashMap::new();
    let (prefix, _) = cwl_metadata.namespaces.iter().next().ok_or_else(|| {
        anyhow!(
            "The CWL file does not have the $namespaces field: {}",
            cwl_file_path.display()
        )
    })?;
    if prefix != "tataki" {
        bail!(
            "The CWL file does not have the 'tataki' namespace: {}",
            cwl_file_path.display()
        );
    }
    for (key, value) in cwl_metadata.entries.iter() {
        if let Some(stripped_key) = key.strip_prefix(&format!("{}:", prefix)) {
            let value = serde_yaml::to_string(value)?;
            let value = value.trim_end();
            extracted_fields.insert(stripped_key.to_string(), value.to_owned());
        }
    }

    Ok(extracted_fields)
}

fn validate_id_and_label(
    parameters: &mut HashMap<String, String>,
    cwl_file_path: &Path,
) -> Result<()> {
    // if both EDAM_ID and LABEL are present, check LABEL are valid.
    if parameters.contains_key(EDAM_ID_KEY) && parameters.contains_key(LABEL_KEY) {
        let id = parameters.get(EDAM_ID_KEY).unwrap();
        let label = parameters.get(LABEL_KEY).unwrap();
        if !edam::EDAM_MAP.correspondence_check_id_and_label(id, label)? {
            info!(
                "The specified edam_id and label do not correspond with each other. Assuming it is a custom label...: edam_id={}, label={}, CWL file={}",
                id,
                label,
                cwl_file_path.display()
            );
        }
    }
    // if both EDAM_ID and LABEL are not present, return error.
    else {
        bail!(
            "The CWL file is missing required fields under the 'tataki' namespace. Please ensure that both 'tataki.{}' and 'tataki.{}' fields are included in the file.: CWL file={}",
            EDAM_ID_KEY,
            LABEL_KEY,
            cwl_file_path.display()
        );
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct InputFile {
    class: String,
    location: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
struct InputFileWrapper {
    #[serde(rename = "input_file")]
    input_file: InputFile,
}

pub fn make_cwl_input_file(target_file_path: PathBuf, temp_dir: &TempDir) -> Result<NamedTempFile> {
    let input_file = InputFile {
        class: "File".to_string(),
        location: target_file_path,
    };

    let input_file_wrapper = InputFileWrapper { input_file };
    let input_file_yaml_str = serde_yaml::to_string(&input_file_wrapper)?;

    // write the content into a temporary file
    // let input_file_path = temp_dir.path().join("input_file.yaml");
    let input_file_struct = Builder::new()
        .prefix("cwl_input_file_")
        .suffix(".yaml")
        .tempfile_in(temp_dir)?;

    let mut file = std::fs::File::create(&input_file_struct)?;
    file.write_all(input_file_yaml_str.as_bytes())?;

    // let input_file_path = PathBuf::from("input_file.yaml").canonicalize()?;

    // Ok(input_file_path.path().to_path_buf())
    Ok(input_file_struct)
}
