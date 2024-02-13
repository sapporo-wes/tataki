mod common;

use std::fs;
use std::path::Path;

use common::{calculate_checksum, tataki};

/*
test cases:
- default
- -f yaml
- -f json --cache-dir
- -o file
- -c conf
- --dry-run
- --quiet
- --verbose
- -c cwl.conf


*/

#[test]
fn output_in_csv() {
    let out = tataki(&["./inputs/toy.sam", "./inputs/toy.fa"], &[]);

    let stdout = out.stdout;

    let mut rdr = csv::Reader::from_reader(stdout.as_bytes());
    let output_records = rdr
        .records()
        .collect::<Result<Vec<_>, csv::Error>>()
        .expect("Failed to parse the output as CSV");

    let mut expected_output_rdr =
        csv::Reader::from_path(Path::new("tests/outputs/expected_output.csv"))
            .expect("Failed to read the expected output file");
    let expected_output_records = expected_output_rdr
        .records()
        .collect::<Result<Vec<_>, csv::Error>>()
        .expect("Failed to parse the expected output as CSV");

    assert_eq!(output_records, expected_output_records);
}

#[test]
fn output_in_yaml() {
    let out = tataki(&["./inputs/toy.sam", "./inputs/toy.fa"], &["-f", "yaml"]);

    let stdout = out.stdout;

    let output_yaml: serde_yaml::Value =
        serde_yaml::from_str(&stdout).expect("Failed to parse the output as YAML");

    let expected_output_str = fs::read_to_string(Path::new("tests/outputs/expected_output.yaml"))
        .expect("Failed to read the expected output file");
    let expected_output_yaml: serde_yaml::Value = serde_yaml::from_str(&expected_output_str)
        .expect("Failed to parse the expected output as YAML");

    assert_eq!(
        output_yaml, expected_output_yaml,
        "The tool's YAML output did not match the expected output."
    );
}

#[test]
fn output_in_json_and_can_keep_cache() {
    let out = tataki(
        &[
            "./inputs/toy.sam",
            "https://github.com/sapporo-wes/tataki/raw/main/tests/inputs/toy.fa",
        ],
        &["--cache-dir", "./cache_dir/", "-f", "json"],
    );

    let stdout = out.stdout;

    let output_json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Failed to parse the output as JSON");

    let expected_output_str = fs::read_to_string(Path::new("tests/outputs/expected_output.json"))
        .expect("Failed to read the expected output file");
    let expected_output_json: serde_json::Value = serde_json::from_str(&expected_output_str)
        .expect("Failed to parse the expected output as JSON");

    assert_eq!(
        output_json, expected_output_json,
        "The tool's JSON output did not match the expected output."
    );

    let stderr = out.stderr;

    let cache_dir = stderr
        .split("Keeping temporary directory:")
        .collect::<Vec<&str>>()[1]
        .split('\n')
        .collect::<Vec<&str>>()[0]
        .trim();
    let cache_dir = Path::new(cache_dir);

    assert!(cache_dir.exists());

    let toy_fa_path = cache_dir.join("toy.fa");

    let toy_fa_sha256 =
        calculate_checksum(toy_fa_path).expect("Failed to calculate the checksum of toy.fa");

    assert_eq!(
        toy_fa_sha256,
        "b2f08eb3c17ade6f4d9933195acc89caf8eefb5b31f89b98f616e9c8e2f9405e"
    );
}

#[test]

fn can_output_to_file() {
    let _ = tataki(
        &["./inputs/toy.sam", "./inputs/toy.fa"],
        &["-o", "./cache_dir/output.csv"],
    );

    let output_path = Path::new("./tests/cache_dir/output.csv");
    assert!(output_path.exists());

    let output_sha256 =
        calculate_checksum(output_path).expect("Failed to calculate the checksum of output.csv");

    assert_eq!(
        output_sha256,
        "81afa82dcd25f408d0f9a1e3ef01f360c158bb3cdbe2e59e8b6f648a34c8972c"
    );
}

#[test]
// Check if the output becomes null when a conf without sam and fasta is specified.
fn can_use_config_file() {
    let out = tataki(
        &["./inputs/toy.sam", "./inputs/toy.fa"],
        &["-c", "./conf/module_order_test.conf"],
    );

    let stdout = out.stdout;

    let mut rdr = csv::Reader::from_reader(stdout.as_bytes());
    let output_records = rdr
        .records()
        .collect::<Result<Vec<_>, csv::Error>>()
        .expect("Failed to parse the output as CSV");

    let mut expected_output_rdr =
        csv::Reader::from_path(Path::new("tests/outputs/expected_output_module_order.csv"))
            .expect("Failed to read the expected output file");
    let expected_output_records = expected_output_rdr
        .records()
        .collect::<Result<Vec<_>, csv::Error>>()
        .expect("Failed to parse the expected output as CSV");

    assert_eq!(output_records, expected_output_records);
}

#[test]
fn can_dry_run() {
    let out = tataki(&["./inputs/toy.sam", "./inputs/toy.fa"], &["--dry-run"]);

    let stdout = out.stdout;

    let output_yaml: serde_yaml::Value =
        serde_yaml::from_str(&stdout).expect("Failed to parse the output as YAML");

    let expected_output_str = fs::read_to_string(Path::new("src/tataki.conf"))
        .expect("Failed to read the expected output file");
    let expected_output_yaml: serde_yaml::Value = serde_yaml::from_str(&expected_output_str)
        .expect("Failed to parse the expected output as YAML");

    assert_eq!(
        output_yaml, expected_output_yaml,
        "The tool's YAML output did not match the expected output."
    );
}

#[test]
fn can_be_quiet() {
    let out = tataki(&["./inputs/toy.sam", "./inputs/toy.fa"], &["--quiet"]);

    let stderr = out.stderr;

    assert_eq!(stderr, "");
}

#[test]
fn can_be_verbose() {
    let out = tataki(&["./inputs/toy.sam", "./inputs/toy.fa"], &["--verbose"]);

    let stderr = out.stderr;

    assert!(stderr.contains("DEBUG"));
}

// TODO
/*
#[test]
fn can_run_cwl() {
    let out = tataki(
        &["./inputs/toy.bam", "./inputs/toy.fa"],
        &["-c", "./inputs/cwl.conf"],
    );

    let stdout = out.stdout;

    let mut rdr = csv::Reader::from_reader(stdout.as_bytes());
    let output_records = rdr
        .records()
        .collect::<Result<Vec<_>, csv::Error>>()
        .expect("Failed to parse the output as CSV");

    let mut expected_output_rdr =
        csv::Reader::from_path(Path::new("tests/outputs/expected_output_cwl.csv"))
            .expect("Failed to read the expected output file");
    let expected_output_records = expected_output_rdr
        .records()
        .collect::<Result<Vec<_>, csv::Error>>()
        .expect("Failed to parse the expected output as CSV");

    assert_eq!(output_records, expected_output_records);
}
*/
