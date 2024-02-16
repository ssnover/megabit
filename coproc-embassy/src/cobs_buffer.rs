use core::cmp::Ordering;

pub struct CobsBuffer<'a, const N: usize> {
    pub data: &'a mut [u8; N],
    write: usize,
    read: usize,
}

#[repr(u8)]
pub enum DecodeError {
    UnknownDecodeErr,
    MsgIncomplete,
    NoBytes,
}

impl<'a, const N: usize> CobsBuffer<'a, N> {
    pub fn new(buf: &'a mut [u8; N]) -> CobsBuffer<'a, N> {
        CobsBuffer {
            data: buf,
            write: 0,
            read: 0,
        }
    }

    #[allow(unused)]
    pub fn available_to_read(&self) -> usize {
        match self.read.cmp(&self.write) {
            Ordering::Equal => 0,
            Ordering::Greater => N - self.read + self.write,
            Ordering::Less => self.write - self.read,
        }
    }

    /// Reads COBS-encoded bytes up to lesser of the available amount or the size of the
    /// provided buffer `buf`.
    #[allow(unused)]
    pub fn read_bytes(&mut self, buf: &mut [u8]) -> usize {
        match self.read.cmp(&self.write) {
            Ordering::Equal => 0,
            Ordering::Greater => {
                // Read through the end of the buffer and then wrap around up to write pointer
                let mut bytes_read = 0;
                for read_idx in self.read..core::cmp::min(N, buf.len()) {
                    // Read to the end of the memory buffer first
                    buf[bytes_read] = self.data[read_idx];
                    bytes_read += 1;
                    self.read += 1;
                }
                if self.read == N {
                    self.read = 0;
                }
                if bytes_read < buf.len() {
                    for read_idx in 0..core::cmp::min(self.write, buf.len() - bytes_read) {
                        buf[bytes_read] = self.data[read_idx];
                        bytes_read += 1;
                        self.read += 1;
                    }
                }
                bytes_read
            }
            Ordering::Less => {
                let mut bytes_read = 0;
                for read_idx in self.read..core::cmp::min(self.write, self.read + buf.len()) {
                    buf[bytes_read] = self.data[read_idx];
                    bytes_read += 1;
                    self.read += 1;
                }
                bytes_read
            }
        }
    }

    /// Decodes a single COBS-encoded section of the buffer and returns the resulting decoded
    /// packet as bytes.
    pub fn read_packet(&mut self, buf: &mut [u8]) -> Result<usize, DecodeError> {
        let mut decoder = cobs::CobsDecoder::new(buf);
        match self.read.cmp(&self.write) {
            Ordering::Equal => Err(DecodeError::NoBytes),
            Ordering::Greater => match decoder.push(&self.data[self.read..]) {
                Err(_n_bytes) => Err(DecodeError::UnknownDecodeErr),
                Ok(Some((n, m))) => {
                    self.read += m;
                    Ok(n)
                }
                Ok(None) => match decoder.push(&self.data[..self.write]) {
                    Err(_) => Err(DecodeError::UnknownDecodeErr),
                    Ok(Some((n, m))) => {
                        self.read = m;
                        Ok(n)
                    }
                    Ok(None) => Err(DecodeError::MsgIncomplete),
                },
            },
            Ordering::Less => match decoder.push(&self.data[self.read..self.write]) {
                Err(_) => Err(DecodeError::UnknownDecodeErr),
                Ok(Some((n, m))) => {
                    self.read += m;
                    Ok(n)
                }
                Ok(None) => Err(DecodeError::MsgIncomplete),
            },
        }
    }

    /// Write COBS-encoded bytes into the buffer (if they're not you'll be sad).
    pub fn write_bytes(&mut self, buf: &[u8]) -> usize {
        let available_without_overwrite = match self.write.cmp(&self.read) {
            Ordering::Equal => {
                // Whole buffer is available to write to
                N
            }
            Ordering::Greater => {
                // Can write to end of buffer and then some before overwrite
                N - self.write + self.read
            }
            Ordering::Less => {
                // Can
                self.read - self.write
            }
        };
        let mut bytes_written = 0;
        if (N - self.write) > buf.len() {
            for write_idx in 0..buf.len() {
                self.data[self.write] = buf[write_idx];
                self.write += 1;
                bytes_written += 1;
            }
        } else {
            for write_idx in 0..(N - self.write) {
                self.data[self.write] = buf[write_idx];
                self.write += 1;
                bytes_written += 1;
            }
            assert_eq!(self.write, N);
            self.write = 0;
            for write_idx in bytes_written..buf.len() {
                self.data[self.write] = buf[write_idx];
                self.write += 1;
                bytes_written += 1;
            }
        }
        if available_without_overwrite < buf.len() {
            self.read = self.write + 1;
            if self.read == N {
                self.read = 0;
            }
        }
        bytes_written
    }
}
