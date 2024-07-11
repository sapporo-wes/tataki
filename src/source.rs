use anyhow::{bail, Result};
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use log::{debug, warn};
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

    /*
    pub fn as_memory(&self) -> Option<&Vec<u8>> {
        match self {
            Self::Memory(m) => Some(m),
            _ => None,
        }
    }
    */

    // if the input is compressed and no_decompress is false, decompress the input, save it into tempfile and return Source::FilePath(tempfile path)
    // if the input is either not compressed or no_decompress is true, return None
    pub fn decompress_into_tempfile_from_filepath_if_needed(
        input_path: &Path,
        options: &InvokeOptions,
        temp_dir: &TempDir,
        cwl_modules_exists: bool,
    ) -> Result<(Option<Self>, CompressedFormat)> {
        let mut file = File::open(input_path)?;

        // read first 100 bytes from reader in order to infer compression format
        let mut buffer = [0; 100];
        let bytes_read = file.read(&mut buffer)?;

        // rewind the reader to the beginning
        file.rewind()?;

        // TODO check
        // if not inferrable, then None.
        let compressed_format: CompressedFormat =
            if let Some(inferred_type) = infer::get(&buffer[..bytes_read]) {
                let extension = inferred_type.extension();
                match extension {
                    "gz" => {
                        // check if the gz file is in BGZF format
                        if is_gzfile_in_bgzf(&buffer) {
                            debug!("Provided input is in BGZF format");
                            CompressedFormat::Bgzf
                        } else {
                            debug!("Provided input is in GZ format");
                            CompressedFormat::GZ
                        }
                    }
                    "bz2" => {
                        debug!("Provided input is in BZ2 format");
                        CompressedFormat::BZ2
                    }
                    _ => {
                        warn!("Provided input is in compressed format not supported by this tool");
                        CompressedFormat::None
                    }
                }
            } else {
                // type was not inferred, return None
                warn!("Provided input is in compressed format not supported by this tool");
                CompressedFormat::None
            };

        // if input is plain text or in BGZF, we are not going to decompress it
        let mut inferred_reader: Box<dyn Read> = match compressed_format {
            CompressedFormat::Bgzf => {
                // if BGZF, save the reader as is
                return Ok((None, CompressedFormat::Bgzf));
            }
            CompressedFormat::GZ => {
                // if GZ, save the GzDecoder reader
                let decoder = GzDecoder::new(file);
                Box::new(decoder)
            }
            CompressedFormat::BZ2 => {
                // if BZ2, save the BzDecoder reader
                let decoder = BzDecoder::new(file);
                Box::new(decoder)
            }
            CompressedFormat::None => {
                // if None, save the reader as is
                return Ok((None, CompressedFormat::None));
            }
        };

        // here the input is compressed and not BGZF
        if options.no_decompress {
            Ok((None, compressed_format))
        } else {
            if cwl_modules_exists && !options.tidy {
                bail!("The `--tidy` options is required when using CWL modules with compressed input files. If you want to treat the input file as is and not decompress it, please use the `--no-decompress` option.");
            }
            let decompressed_tempfile = Self::read_numrecords_save_into_tempfile(
                &mut inferred_reader,
                options,
                temp_dir,
                false,
            )?;

            Ok((Some(decompressed_tempfile), compressed_format))
        }
    }

    // if the input from stdin is compressed and no_decompress is false, decompress the input, save it into tempfile and return Source::FilePath(tempfile path)
    // if the input from stdin is either not compressed or no_decompress is true, save it into tempfile and return Source::FilePath(tempfile path)
    pub fn convert_into_tempfile_from_stdin(
        options: &InvokeOptions,
        temp_dir: &TempDir,
    ) -> Result<(Self, CompressedFormat)> {
        let stdin = std::io::stdin();
        let handle = stdin.lock();

        let mut onetime_reader = OnetimeRewindableReader::new(handle);

        // read first 100 bytes from reader in order to infer compression format
        let mut buffer = [0; 100];
        let bytes_read = onetime_reader.read(&mut buffer)?;

        // rewind the reader to the beginning
        onetime_reader.rewind()?;

        let mut is_bgzf = false;

        // if not inferrable, then None.
        // TODO refactor into a function
        let compressed_format: CompressedFormat =
            if let Some(inferred_type) = infer::get(&buffer[..bytes_read]) {
                let extension = inferred_type.extension();
                match extension {
                    "gz" => {
                        // check if the gz file is in BGZF format
                        if is_gzfile_in_bgzf(&buffer) {
                            is_bgzf = true;
                            debug!("Provided input is in BGZF format");
                            CompressedFormat::Bgzf
                        } else {
                            debug!("Provided input is in GZ format");
                            CompressedFormat::GZ
                        }
                    }
                    "bz2" => {
                        debug!("Provided input is in BZ2 format");
                        CompressedFormat::BZ2
                    }
                    _ => {
                        warn!("Provided input is in compressed format not supported by this tool");
                        CompressedFormat::None
                    }
                }
            } else {
                // type was not inferred, return None
                warn!("Provided input is in compressed format not supported by this tool");
                CompressedFormat::None
            };

        // use the reader as is if no_decompress is true
        let mut inferred_reader: Box<dyn Read> = if options.no_decompress {
            Box::new(onetime_reader)
        } else {
            match compressed_format {
                CompressedFormat::Bgzf => {
                    // if BGZF, save the reader as is
                    Box::new(onetime_reader)
                }
                CompressedFormat::GZ => {
                    // if GZ, save the GzDecoder reader
                    let decoder = GzDecoder::new(onetime_reader);
                    Box::new(decoder)
                }
                CompressedFormat::BZ2 => {
                    // if BZ2, save the BzDecoder reader
                    let decoder = BzDecoder::new(onetime_reader);
                    Box::new(decoder)
                }
                CompressedFormat::None => {
                    // if None, save the reader as is
                    Box::new(onetime_reader)
                }
            }
        };

        let tempfile_from_stdin = Self::read_numrecords_save_into_tempfile(
            &mut inferred_reader,
            options,
            temp_dir,
            is_bgzf || options.no_decompress,
        )?;

        Ok((tempfile_from_stdin, compressed_format))
    }

    fn read_numrecords_save_into_tempfile<R: Read>(
        reader: &mut R,
        options: &InvokeOptions,
        temp_dir: &TempDir,
        is_binary: bool,
    ) -> Result<Self> {
        // create a writer for tempfile
        let mut tempfile = NamedTempFile::new_in(temp_dir)?;

        // if the input is binary, such as BGZF, read (100 * num_records) bytes and save it into tempfile
        if is_binary {
            let total_bytes_copied: u64 = if options.tidy {
                std::io::copy(reader, &mut tempfile)?
            } else {
                let bytes_to_copy = 100 * options.num_records;
                let mut limited_src = reader.take(bytes_to_copy as u64);
                std::io::copy(&mut limited_src, &mut tempfile)?
            };
            debug!("Bytes read from STDIN: {}", total_bytes_copied);
        }
        // if not in binary, read the first 4 * num_records lines (plus header) and save it into tempfile
        else {
            let mut bufreader = BufReader::new(reader);

            let mut line_buffer = String::new();
            let mut count = 0;
            let mut header_count = 0;
            let numlines_to_read = 4 * options.num_records;
            let max_header_lines = 20;
            while options.tidy || count < numlines_to_read {
                line_buffer.clear();

                let bytes_read = bufreader.read_line(&mut line_buffer)?;
                if bytes_read == 0 {
                    break;
                }

                // if the line read is presumably a comment line, do not count it as a read line until the count reaches the max_header_lines
                if (line_buffer.starts_with('#') || line_buffer.starts_with('@'))
                    && header_count < max_header_lines
                {
                    header_count += 1;
                } else {
                    count += 1
                }

                // write line into tempfile
                tempfile.write_all(line_buffer.as_bytes())?;
            }
        }

        Ok(Self::TempFile(tempfile))
    }
}

// check if the gz file is particulary in BGZF format
fn is_gzfile_in_bgzf(header_buffer: &[u8]) -> bool {
    // check if the header is BGZF

    // check if the header is long enough
    let header_buffer_length = header_buffer.len() >= 15;

    // GZ Flag = 4, means that there is an extra field
    let flag = header_buffer[3] == 0x04;

    // SI1 field of the extra field is 66
    let si1 = header_buffer[12] == 0x42;

    // SI2 field of the extra field is 67
    let si2 = header_buffer[13] == 0x43;

    // subfield length in the extra field is 2
    let slen = header_buffer[14] == 0x02;

    header_buffer_length && flag && si1 && si2 && slen
}

#[derive(Debug)]
pub enum CompressedFormat {
    Bgzf,
    GZ,
    BZ2,
    None,
}
