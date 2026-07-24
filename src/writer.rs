use std::io::{self, Write};

/// Trait for implementing the `Write` trait,
/// which can record the total number of bytes written.
pub(crate) trait CountingWrite: Write {
  fn written_bytes(&self) -> usize;
}

pub(crate) struct CountingWriter<W> {
  pub inner: W,
  bytes_count: usize,
}

impl<W> CountingWriter<W> {
  pub fn new(inner: W) -> Self {
    Self {
      inner,
      bytes_count: 0,
    }
  }
}

impl<W: Write> CountingWrite for CountingWriter<W> {
  fn written_bytes(&self) -> usize {
    self.bytes_count
  }
}

impl<W: Write> Write for CountingWriter<W> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let n = self.inner.write(buf)?;
    self.bytes_count += n;
    Ok(n)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.inner.flush()
  }
}
