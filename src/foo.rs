use anyhow::{anyhow, bail, ensure, Context, Result};
use clap::{Parser, Subcommand};
use noodles::bam;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Read, Seek};
use std::path::PathBuf;

fn hoge() -> Result<()> {
    todo!();
    let is_binary: bool;

    let mut file = File::open(&opts.input_file_path)?;

    let first_line = read_first_line(&file);
    match first_line {
        // the input file contains valid UTF-8
        Some(result) => match result {
            Ok(line) => {
                is_binary = false;
            }
            Err(e) => {
                is_binary = true;
            }
        },
        None => {
            bail!("the input file is empty");
        }
    }

    file.rewind()?;
    // let reader = BufReader::new(&file);

    let input_file_path_cow_str = (&opts.input_file_path).to_string_lossy();

    match is_binary {
        true => {
            if is_bam_file_tidy(&opts.input_file_path).unwrap_or(false) {
                stdout_result(&input_file_path_cow_str, "BAM file, Blocked GNU Zip Format");
                return Ok(());
            }
            if is_fastq_gz(&opts.input_file_path).unwrap_or(false) {
                stdout_result(&input_file_path_cow_str, "gzip compressed fastq file");
                return Ok(());
            }
            stdout_result(&input_file_path_cow_str, "binary file")
        }
        false => {
            if is_fasta(&opts.input_file_path).unwrap_or(false) {
                stdout_result(&input_file_path_cow_str, "fasta file");
                return Ok(());
            }
            if is_fastq(&opts.input_file_path).unwrap_or(false) {
                stdout_result(&input_file_path_cow_str, "fastq file");
                return Ok(());
            }
            match is_bed(&opts.input_file_path).unwrap_or(None) {
                Some(col_count) => {
                    let file_type_str = format!("{} column BED file", col_count);
                    stdout_result(&input_file_path_cow_str, &file_type_str);
                    return Ok(());
                }
                None => {}
            }
            stdout_result(&input_file_path_cow_str, "ascii file")
        }
    }

    Ok(())
}

fn stdout_result(path: &Cow<str>, file_type: &str) {
    todo!();
    println!("{}:\t{}", path, file_type);
}

fn is_bam_file_tidy(path: &PathBuf) -> Result<bool> {
    todo!();
    let mut reader = File::open(path).map(bam::Reader::new)?;

    let header = reader.read_header()?;
    reader.read_reference_sequences()?;

    Ok(true)
}

fn is_bed(path: &PathBuf) -> Result<Option<usize>> {
    todo!();
    let contents = read_whole_file_into_string(path)?;

    // let re = Regex::new(r"(browser.*\n|track.*\n|\#.*\n)")?;
    let re_comment_header = Regex::new(r"(browser|track|#).*")?;
    let re_whitespace = Regex::new(r"^[ \t]+")?;
    let re_blank = Regex::new(r"^$")?;
    let re_col_list: Vec<Regex> = vec![
        Regex::new(r"^[[:word:]]{1,255}\t\d{1,20}\t\d{1,20}$")?,
        Regex::new(r"^[[:word:]]{1,255}\t\d{1,20}\t\d{1,20}\t[\x20-\x7e]{1,255}$")?,
        Regex::new(r"^[[:word:]]{1,255}\t\d{1,20}\t\d{1,20}\t[\x20-\x7e]{1,255}\t\d{0,4}$")?,
        Regex::new(
            r"^[[:word:]]{1,255}\t\d{1,20}\t\d{1,20}\t[\x20-\x7e]{1,255}\t\d{0,4}\t[\-\+\.]$",
        )?,
        Regex::new(
            r"^[[:word:]]{1,255}\t\d{1,20}\t\d{1,20}\t[\x20-\x7e]{1,255}\t\d{0,4}\t[\-\+\.]\t\d{1,20}$",
        )?,
        Regex::new(
            r"^[[:word:]]{1,255}\t\d{1,20}\t\d{1,20}\t[\x20-\x7e]{1,255}\t\d{0,4}\t[\-\+\.]\t\d{1,20}\t\d{1,20}$",
        )?,
        Regex::new(
            r"^[[:word:]]{1,255}\t\d{1,20}\t\d{1,20}\t[\x20-\x7e]{1,255}\t\d{0,4}\t[\-\+\.]\t\d{1,20}\t\d{1,20}\t(\d{1,3},\d{1,3},\d{1,3}|0)$",
        )?,
        Regex::new(
            r"^[[:word:]]{1,255}\t\d{1,20}\t\d{1,20}\t[\x20-\x7e]{1,255}\t\d{0,4}\t[\-\+\.]\t\d{1,20}\t\d{1,20}\t(\d{1,3},\d{1,3},\d{1,3}|0)\t\d{1,20}$",
        )?,
        Regex::new(
            r"^[[:word:]]{1,255}\t\d{1,20}\t\d{1,20}\t[\x20-\x7e]{1,255}\t\d{0,4}\t[\-\+\.]\t\d{1,20}\t\d{1,20}\t(\d{1,3},\d{1,3},\d{1,3}|0)\t\d{1,20}\t(\d+,){1,20}\d+,?$",
        )?,
        Regex::new(
            r"^[[:word:]]{1,255}\t\d{1,20}\t\d{1,20}\t[\x20-\x7e]{1,255}\t\d{0,4}\t[\-\+\.]\t\d{1,20}\t\d{1,20}\t(\d{1,3},\d{1,3},\d{1,3}|0)\t\d{1,20}\t(\d+,){1,20}\d+,?\t(\d+,){1,20}\d+,?$",
        )?,
    ];

    let mut index_re: Option<usize> = None;

    for line in contents.lines() {
        if re_comment_header.is_match(line)
            || re_blank.is_match(line)
            || re_whitespace.is_match(line)
        {
        } else {
            match index_re {
                Some(index) => {
                    if !re_col_list[index].is_match(line) {
                        return Ok(None);
                    }
                }
                None => {
                    for (index, re) in re_col_list.iter().enumerate() {
                        if re.is_match(line) {
                            index_re = Some(index);
                            break;
                        }
                    }
                    if index_re.is_none() {
                        return Ok(None);
                    }
                }
            }
        }
    }

    Ok(Some(index_re.unwrap() + 3usize))
}

fn is_fasta(path: &PathBuf) -> Result<bool> {
    todo!();
    let contents = read_whole_file_into_string(path)?;

    let re = Regex::new(r"^(>.*\n([^>\s]+\n)+)+\n?")?;
    let mat = re.find(&contents);

    match mat {
        Some(m) => {
            if contents.len() == m.end() {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        None => Ok(false),
    }
}

fn is_fastq(path: &PathBuf) -> Result<bool> {
    todo!();
    let contents = read_whole_file_into_string(path)?;

    validate_fastq(&contents)
}

fn is_fastq_gz(path: &PathBuf) -> Result<bool> {
    todo!();
    let decoded_contents = gzip_decode_reader(path)?;

    validate_fastq(&decoded_contents)
}

fn validate_fastq(contents: &str) -> Result<bool> {
    todo!();
    let re = Regex::new(r"^@.*\n[ATGCNatgcn]*\n+.*\n.*\n")?;

    let mut content_lines = contents.lines();

    loop {
        let next_4lines = content_lines.by_ref().take(4);
        let next_4lines_string = next_4lines
            .map(|x| x.to_string() + "\n")
            .collect::<String>();

        if next_4lines_string.trim_end().is_empty() {
            break;
        }

        if !re.is_match(&next_4lines_string) {
            return Ok(false);
        }
    }

    Ok(true)
}

fn gzip_decode_reader(path: &PathBuf) -> Result<String> {
    todo!();
    let mut buf = Vec::new();
    let mut file = File::open(path)?;
    file.read_to_end(&mut buf)?;

    let mut gz = GzDecoder::new(&buf[..]);

    let mut s = String::new();
    gz.read_to_string(&mut s)?;

    Ok(s)
}

fn is_bam_file_with_nom(file: &mut File) -> Result<bool> {
    todo!();

    // read_file
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    println!("{:?}", buf);

    // let (input, header) = read_bgzf_header(&buf)?;

    println!();
    // println!("{:?}", &input);
    // println!("{:?}", &header);

    Ok(true)
}

fn read_bgzf_header<'a>(input: &'a [u8]) -> IResult<&'a [u8], BGZFFileHeader> {
    todo!();

    let (input, _) = tag(b"\x1f")(input)?;
    let (input, _) = tag(b"\x8b")(input)?;
    let (input, _) = tag(b"\x08")(input)?;
    let (input, _) = tag(b"\x04")(input)?;
    let (input, _) = tag([0u8; 4])(input)?;
    let (input, _) = tag(b"\x00")(input)?;
    let (input, _) = tag(b"\xff")(input)?;
    let (input, _) = tag(b"\x06\x00")(input)?;
    let (input, _) = tag(b"\x42")(input)?;
    let (input, _) = tag(b"\x43")(input)?;
    let (input, _) = tag(b"\x02\x00")(input)?;
    let (input, bsize) = le_u16(input)?;
    let (input, cdata) = u8(input)?;
    let (input, crc32) = le_u32(input)?;
    let (input, i_size) = le_u32(input)?;

    Ok((
        input,
        BGZFFileHeader {
            bsize,
            cdata,
            crc32,
            i_size,
        },
    ))
}

fn read_first_line(file: &File) -> Option<Result<String, Error>> {
    todo!();
    let reader = BufReader::new(file);

    reader.lines().next()
}

fn read_whole_file_into_string(path: &PathBuf) -> Result<String> {
    todo!();
    let mut file = File::open(path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}

#[derive(Debug, Clone)]
struct BGZFFileHeader {
    // /// gzip IDentifier1
    // id1: u8,
    // /// gzip IDentifier2
    // id2: u8,
    // /// gzip Compression Method
    // cm: u8,
    // /// gzip FLaGs
    // flg: u8,
    // ///gzip Modification TIME
    // mtime: u32,
    // /// gzip eXtra FLags
    // xfl: u8,
    // /// gzip Operating System
    // os: u8,
    // /// gzip eXtra LENgth
    // xlen: u16,
    // /// Subfield Identifier1
    // si1: u8,
    // /// Subfield Identifier2
    // si2: u8,
    // /// Subfield LENgth
    // slen: u16,
    /// total Block SIZE minus 1
    bsize: u16,
    /// Compressed DATA by zlib::deflate()
    cdata: u8,
    /// CRC-32
    crc32: u32,
    /// Input SIZE
    i_size: u32,
}

fn stdout_license() {
    todo!();
    println!("Copyright [2022] [@suecharo]");
    println!("This software is released under Apache License 2.0");
}
