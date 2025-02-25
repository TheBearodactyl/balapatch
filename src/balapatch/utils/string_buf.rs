use std::io::{Cursor, Read, Write};

/// A struct that provides in-memory string I/O capabilities
///
/// Implements `Write` for building up content and provides
/// `reader()` to get a `Read` implementor for the content
pub struct StringBuf {
    buffer: Vec<u8>,
}

impl StringBuf {
    /// Creates a new empty StringIO buffer
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Creates a reader that provides read access to the current buffer contents
    ///
    /// The reader will read from the start of the buffer and cannot modify it
    pub fn reader(&self) -> impl Read + '_ {
        Cursor::new(&self.buffer)
    }

    /// Consumes the StringIO and returns the contents as a String
    ///
    /// # Errors
    /// Returns an error if the buffer contains invalid UTF-8
    pub fn into_string(self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.buffer)
    }

    /// Returns the current contents as a String reference
    ///
    /// # Errors
    /// Returns an error if the buffer contains invalid UTF-8
    pub fn as_string(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.buffer)
    }
}

impl Write for StringBuf {
    /// Appends data to the buffer
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);

        Ok(buf.len())
    }

    /// No-op flush implementation
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Default for StringBuf {
    fn default() -> Self {
        Self::new()
    }
}