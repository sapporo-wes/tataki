use anyhow::{bail, Result};
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use tempfile::{NamedTempFile, TempDir};

use crate::buffered_read_seek::OnetimeRewindableReader;
use crate::module::InvokeOptions;

static STDIN_IS_USED: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub enum Source {
    FilePath(PathBuf),
    TempFile(NamedTempFile),
    Stdin,
    Memory(Vec<u8>),
}

impl FromStr for Source {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "-" => {
                if STDIN_IS_USED.load(std::sync::atomic::Ordering::Relaxed) {
                    bail!("Only one stdin source is allowed");
                }
                STDIN_IS_USED.store(true, std::sync::atomic::Ordering::Relaxed);
                Ok(Self::Stdin)
            }
            _ => Ok(Self::FilePath(PathBuf::from(s))),
        }
    }
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FilePath(p) => p.display().fmt(f),
            Self::TempFile(t) => t.path().display().fmt(f),
            Self::Stdin => write!(f, "STDIN"),
            Self::Memory(_) => write!(f, "MEMORY"), // non expected case
        }
    }
}

impl Source {
    pub fn as_path(&self) -> Option<&PathBuf> {
        match self {
            Self::FilePath(p) => Some(p),
            _ => None,
        }
    }

    pub fn as_memory(&self) -> Option<&Vec<u8>> {
        match self {
            Self::Memory(m) => Some(m),
            _ => None,
        }
    }

    pub fn convert_into_tempfile_from_file(
        input_path: &Path,
        options: &InvokeOptions,
        temp_dir: &TempDir,
    ) -> Result<Self> {
        let file = File::open(input_path)?;
        Self::convert_into_tempfile(file, options, temp_dir)
    }

    pub fn convert_into_tempfile_from_stdin(
        options: &InvokeOptions,
        temp_dir: &TempDir,
    ) -> Result<Self> {
        let stdin = std::io::stdin();
        let handle = stdin.lock();
        Self::convert_into_tempfile(handle, options, temp_dir)
    }

    fn convert_into_tempfile<R: Read>(
        readable: R,
        options: &InvokeOptions,
        temp_dir: &TempDir,
    ) -> Result<Self> {
        // create a writer for tempfile
        let mut tempfile = NamedTempFile::new_in(temp_dir)?;
        // let mut writer = BufWriter::new(&tempfile);

        let mut tmp_reader = OnetimeRewindableReader::new(readable);

        // read first 10 bytes from reader in order to infer compression format
        let mut buffer = [0; 100];
        let bytes_read = tmp_reader.read(&mut buffer)?;

        println!("buffer: {:?}", String::from_utf8(buffer.to_vec()));

        // rewind the reader to the beginning
        tmp_reader.rewind()?;

        // let mut buffer = [0; 10];
        // let bytes_read = tmp_reader.read(&mut buffer)?;
        // println!("buffer: {:?}", String::from_utf8(buffer.to_vec()));

        // let bytes_read = tmp_reader.read(&mut buffer)?;
        // println!("buffer: {:?}", String::from_utf8(buffer.to_vec()));

        // let bytes_read = tmp_reader.read(&mut buffer)?;
        // println!("buffer: {:?}", String::from_utf8(buffer.to_vec()));
        // std::process::exit(1);

        let reader: Box<dyn Read> = if options.no_decompress {
            Box::new(tmp_reader)
        } else if let Some(inferred_type) = infer::get(&buffer[..bytes_read]) {
            let extension = inferred_type.extension();
            match extension {
                "gz" => {
                    // TODO BAMのBGZFがここで誤認されないようにする
                    let decoder = GzDecoder::new(tmp_reader);
                    println!("gz!!!");
                    Box::new(decoder)
                }
                "bz2" => {
                    let decoder = BzDecoder::new(tmp_reader);
                    println!("bz2!!!");
                    Box::new(decoder)
                }
                _ => {
                    panic!("Unexpected infer result");
                }
            }
        } else {
            Box::new(tmp_reader)
        };

        let mut bufreader = BufReader::new(reader);

        // TODO tmpに保存する行数を4倍くらいにする。memo参照
        let mut line_buffer = String::new();
        let mut count = 0;
        while options.tidy || count < options.num_records {
            line_buffer.clear();
            let bytes_read = bufreader.read_line(&mut line_buffer)?;
            if bytes_read == 0 {
                break;
            }
            count += 1;
            // write line into tempfile
            tempfile.write_all(line_buffer.as_bytes())?;
        }

        // for (count, line) in bufreader.lines().enumerate() {
        //     if !options.tidy && count >= options.num_records {
        //         break;
        //     }
        //     let line = line?;
        //     // write line into tempfile
        //     tempfile.write_all(line.as_bytes())?;

        //     // writeln!(writer, "{}", line)?;
        // }

        println!("tempfile: {:?}", tempfile.path());

        Ok(Self::TempFile(tempfile))
    }
}
