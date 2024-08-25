// use sha2::Sha256;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub struct Out {
    pub stdout: String,
    pub stderr: String,
}

pub fn tataki(targets: &[&str], options: &[&str]) -> Out {
    let mut cmd = assert_cmd::Command::cargo_bin("tataki").expect("Failed to find 'tataki' binary");
    cmd.current_dir("tests/");
    let hoge = cmd.args(targets).args(options).assert().success();

    Out {
        stdout: String::from_utf8_lossy(&hoge.get_output().stdout).to_string(),
        stderr: String::from_utf8_lossy(&hoge.get_output().stderr).to_string(),
    }
}

pub fn calculate_checksum<P>(path: P) -> io::Result<String>
where
    P: AsRef<Path>,
{
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;

    hasher.update(&buffer);
    let result = hasher.finalize();

    Ok(format!("{:x}", result))
}

// check for the existence of a cache_dir directory and create it if it does not exist
pub fn check_and_create_cache_dir() -> io::Result<()> {
    let cache_dir_path = Path::new("tests/cache_dir");
    if !cache_dir_path.exists() {
        std::fs::create_dir_all(cache_dir_path)?;
    }

    Ok(())
}

// whether the current environment is M1 Mac
#[allow(dead_code)]
pub fn is_running_on_m1_mac() -> bool {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    os == "macos" && arch == "aarch64"
}
