use anyhow::{ensure, Result};
use chrono::Local;
use std::path::PathBuf;
use std::time;
use std::{fs::File, io::Write};
use tempfile::TempDir;
use url::Url;

pub fn create_temporary_dir(cache_dir: &Option<PathBuf>) -> Result<TempDir> {
    let prefix = format!("tataki_{}_", Local::now().format("%Y-%m%d-%H%M%S"));

    if let Some(cache_dir_path) = cache_dir {
        // create a temporary directory in the specified directory if `--cache-dir` is specified .
        let temp_dir = tempfile::Builder::new()
            .prefix(&prefix)
            .tempdir_in(cache_dir_path)?;
        Ok(temp_dir)
    } else {
        // create a temporary directory in /tmp
        let temp_dir = tempfile::Builder::new().prefix(&prefix).tempdir()?;
        Ok(temp_dir)
    }
}

pub fn download_from_url(url: &Url, temp_dir: &TempDir) -> Result<PathBuf> {
    // timeout in 60 * 60 seconds
    let client = reqwest::blocking::Client::builder()
        .timeout(time::Duration::from_secs(3600))
        .build()?;
    let response = client.get(url.as_ref()).send()?;
    let status = response.status();
    let response_bytes = response.bytes()?;

    ensure!(
        status.is_success(),
        "Failed to download from {} with status code {}",
        url.as_str(),
        status
    );

    // write the content of the response to a temporary file
    let file_name = &url
        .path_segments()
        .and_then(|segments| segments.last())
        .unwrap_or("downloaded_file");
    let file_path = temp_dir.path().join(file_name);
    let mut file = File::create(&file_path)?;
    file.write_all(&response_bytes)?;

    Ok(file_path)
}
