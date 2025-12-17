mod common;

use std::fs;
use std::path::Path;

use common::{calculate_checksum, check_and_create_cache_dir, tataki};

/*
test cases:
1. default
2. -f yaml
3. -f json --cache-dir
4. -o file
5. -c conf
6. --dry-run
7. --quiet
8. --verbose
9. -c cwl.conf

new test cases involving new features:
10. --num-records <LINES>
11. --tidy
12. read STDIN
    1. --num-records - (check #lines of a tempfile)
    2. gzipped stdin
    3. conflicts w/ `--tidy`
    4. conflictl w/ cwl extension mode
13. read compressed file
    1. types
        1. read gzipped file
        2. read bz2 file
        3. read bzgf file
    2. --no-decompress
    3. --tidy
    4. conflicts w/ cwl extension mode


13. --no-decompress
*/

#[test]
// 1. default
fn output_in_csv() {
    check_and_create_cache_dir().expect("Failed to create the cache directory");

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
// 2. -f yaml
fn output_in_yaml() {
    check_and_create_cache_dir().expect("Failed to create the cache directory");

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
// 3. -f json --cache-dir
fn output_in_json_and_can_keep_cache() {
    check_and_create_cache_dir().expect("Failed to create the cache directory");

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
        "e97c184fd81943e69d750a0e6bad5248c927a7430cb44598221dbf0263356078"
    );
}

#[test]
// 4. -o file
fn can_output_to_file() {
    check_and_create_cache_dir().expect("Failed to create the cache directory");

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
        "b60ee9b2d903fa08aa29575e2a1b719ce3b678b374e2fb57ee64355e10534840"
    );
}

#[test]
// 5. -c conf
// Check if the output becomes null when a conf without sam and fasta is specified.
fn can_use_config_file() {
    check_and_create_cache_dir().expect("Failed to create the cache directory");

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
// 6. --dry-run
fn can_dry_run() {
    check_and_create_cache_dir().expect("Failed to create the cache directory");

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
// 7. --quiet
fn can_be_quiet() {
    check_and_create_cache_dir().expect("Failed to create the cache directory");

    let out = tataki(&["./inputs/toy.sam", "./inputs/toy.fa"], &["--quiet"]);

    let stderr = out.stderr;

    assert_eq!(stderr, "");
}

#[test]
// 8. --verbose
fn can_be_verbose() {
    check_and_create_cache_dir().expect("Failed to create the cache directory");

    let out = tataki(&["./inputs/toy.sam", "./inputs/toy.fa"], &["--verbose"]);

    let stderr = out.stderr;

    assert!(stderr.contains("DEBUG"));
}

#[test]
// 9. -c cwl.conf
fn can_run_cwl() {
    check_and_create_cache_dir().expect("Failed to create the cache directory");

    let out = tataki(
        &["./inputs/toy.py", "./inputs/toy.fa"],
        &["-c", "./conf/run_cwl_test.conf"],
    );

    let stdout = out.stdout;

    let mut rdr = csv::Reader::from_reader(stdout.as_bytes());
    let output_records = rdr
        .records()
        .collect::<Result<Vec<_>, csv::Error>>()
        .expect("Failed to parse the output as CSV");

    let mut expected_output_rdr =
        csv::Reader::from_path(Path::new("tests/outputs/expected_output_run_cwl.csv"))
            .expect("Failed to read the expected output file");
    let expected_output_records = expected_output_rdr
        .records()
        .collect::<Result<Vec<_>, csv::Error>>()
        .expect("Failed to parse the expected output as CSV");

    assert_eq!(output_records, expected_output_records);
}

#[test]
// 10. --num-records <LINES>
// Check if tataki only reads a single records. The second line of the input file is in abnormal format. If tataki reads more than one record, this assert fails.
fn can_limit_the_number_of_output_records() {
    check_and_create_cache_dir().expect("Failed to create the cache directory");

    let out = tataki(&["./inputs/toy_invalid_flag.sam"], &["--num-records", "1"]);

    let stdout = out.stdout;

    let mut rdr = csv::Reader::from_reader(stdout.as_bytes());
    let output_records = rdr
        .records()
        .collect::<Result<Vec<_>, csv::Error>>()
        .expect("Failed to parse the output as CSV");

    let mut expected_output_rdr =
        csv::Reader::from_path(Path::new("tests/outputs/expected_output_num_records.csv"))
            .expect("Failed to read the expected output file");
    let expected_output_records = expected_output_rdr
        .records()
        .collect::<Result<Vec<_>, csv::Error>>()
        .expect("Failed to parse the expected output as CSV");

    assert_eq!(output_records, expected_output_records);
}

#[test]
// 11. --tidy
// Check if tataki attempt to read the whole lines of the input file and fail when parsing the line right after `--num-records` lines.
fn can_read_entirety_of_input_file() {
    check_and_create_cache_dir().expect("Failed to create the cache directory");

    let out = tataki(&["./inputs/toy.sam", "./inputs/toy.fa"], &["--tidy"]);

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
