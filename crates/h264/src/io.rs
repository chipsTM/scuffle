/// A wrapper around a [`std::io::Read`] or [`std::io::Write`] that automatically inserts or removes
/// emulation prevention bytes, when reading or writing respectively.
pub struct EmulationPreventionIo<I> {
    inner: I,
    zero_count: u8,
}

impl<I> EmulationPreventionIo<I> {
    /// Creates a new `EmulationPrevention` wrapper around the given [`std::io::Read`] or [`std::io::Write`].
    /// This should be a buffered reader or writer because we will only read or write one byte at a time.
    /// If the underlying io is not buffered this will result in poor performance.
    pub fn new(inner: I) -> Self {
        Self { inner, zero_count: 0 }
    }
}

impl<I: std::io::Write> std::io::Write for EmulationPreventionIo<I> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for &byte in buf {
            if self.zero_count >= 2 && (byte <= 0x03) {
                self.inner.write_all(&[0x3])?;
                self.zero_count = 0;
            }

            self.inner.write_all(&[byte])?;
            if byte == 0x00 {
                self.zero_count += 1;
            } else {
                self.zero_count = 0;
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl<I: std::io::Read> std::io::Read for EmulationPreventionIo<I> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut read_size = 0;
        let mut one_byte = [0; 1];
        while buf.len() > read_size {
            let size = self.inner.read(&mut one_byte)?;
            if size == 0 {
                break;
            }

            let byte = one_byte[0];
            match byte {
                0x03 if self.zero_count >= 2 => {
                    self.zero_count = 0;
                    continue;
                }
                0x00 => {
                    self.zero_count += 1;
                }
                _ => {
                    self.zero_count = 0;
                }
            }

            buf[read_size] = byte;
            read_size += 1;
        }

        Ok(read_size)
    }
}
