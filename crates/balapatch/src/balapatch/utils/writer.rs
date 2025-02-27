use indicatif::MultiProgress;
use std::io::{self, Write};
use std::sync::Arc;

pub struct MultiProgressWriter {
    mp: Arc<MultiProgress>,
}

impl MultiProgressWriter {
    pub fn _new(mp: Arc<MultiProgress>) -> Self {
        Self { mp }
    }
}

impl Write for MultiProgressWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Write to the underlying MultiProgress
        self.mp.println(String::from_utf8_lossy(buf))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // No-op, as MultiProgress doesn't need flushing
        Ok(())
    }
}
