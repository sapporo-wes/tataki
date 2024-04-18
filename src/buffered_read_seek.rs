use anyhow::Result;
use flate2::read::{GzDecoder, ZlibDecoder};
use std::io::{self, Cursor, Read, Seek, SeekFrom};

pub struct OnetimeRewindableReader<R: Read> {
    inner: R,
    buffer: Option<Cursor<Vec<u8>>>,
    has_rewound: bool,
}

impl<R: Read> OnetimeRewindableReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            buffer: None,
            has_rewound: false,
        }
    }
}

impl<R: Read> Read for OnetimeRewindableReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Some(ref mut buffer) = self.buffer {
            // 既にバッファがある場合はそこから読み取る
            let count = buffer.read(buf)?;
            if count == 0 {
                // バッファが空になった場合は、元の入力から読み取る
                let innercount = self.inner.read(buf);
                innercount
            } else {
                Ok(count)
            }
        } else {
            // 初回の読み取りでバッファを準備
            let mut temp_buffer = vec![0; buf.len()];
            let count = self.inner.read(&mut temp_buffer)?;
            if count > 0 {
                // 読み取ったデータをバッファとして保存
                self.buffer = Some(Cursor::new(temp_buffer));
                self.read(buf)
            } else {
                Ok(0) // EOF
            }
        }
    }
}

impl<R: Read> Seek for OnetimeRewindableReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(0) => {
                if self.has_rewound {
                    Err(io::Error::new(io::ErrorKind::Other, "Can only rewind once"))
                } else {
                    if let Some(ref mut buffer) = self.buffer {
                        buffer.seek(SeekFrom::Start(0))?;
                    }
                    self.has_rewound = true;
                    Ok(0)
                }
            }
            _ => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Only seeking to start is supported",
            )),
        }
    }
}

/*
// 構造体の定義
pub struct OnetimeRewindableReader<R: Read> {
    inner: R,
    buffer: Cursor<Vec<u8>>,
    eof: bool,
}

impl<R: Read> OnetimeRewindableReader<R> {
    // 新しいBufferedReadSeekを作成
    pub fn new(inner: R) -> OnetimeRewindableReader<R> {
        OnetimeRewindableReader {
            inner,
            buffer: Cursor::new(Vec::new()),
            eof: false,
        }
    }

    // 内部バッファから読み出す、もしくは必要に応じて入力ソースから読み込む
    fn fill_buffer(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.eof {
            return Ok(0);
        }

        if self.buffer.position() as usize == self.buffer.get_ref().len() {
            // println!("buffer filled");
            // println!("buffer prev: {:?}", self.buffer.get_ref());
            let mut temp_buf = vec![0; buf.len()];
            let bytes_read = self.inner.read(&mut temp_buf)?;
            if bytes_read == 0 {
                self.eof = true; // ファイルの終わりに達したのでフラグをセット
                return Ok(0);
            }
            self.buffer
                .get_mut()
                .extend_from_slice(&temp_buf[..bytes_read]);
            // self.buffer.set_position(0);
            // println!("buffer afte: {:?}", self.buffer.get_ref());
            // println!("position: {:?}", self.buffer.position());
        }
        self.buffer.read(buf)
    }
}

// Readトレイトの実装
impl<R: Read> Read for OnetimeRewindableReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.fill_buffer(buf)
    }
}

// Seekトレイトの実装
impl<R: Read> Seek for OnetimeRewindableReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.eof = false;
        self.buffer.seek(pos)
    }
}
*/
