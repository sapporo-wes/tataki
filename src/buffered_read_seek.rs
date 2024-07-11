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
            // read from the buffer if it exists
            let count = buffer.read(buf)?;
            if count == 0 {
                // if the buffer is empty, read from the original input
                self.inner.read(buf)
            } else {
                Ok(count)
            }
        } else {
            // Prepare the buffer for the first read
            let mut temp_buffer = vec![0; buf.len()];
            let count = self.inner.read(&mut temp_buffer)?;
            if count > 0 {
                // save the read data as a buffer
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
