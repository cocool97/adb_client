use std::io::{self, Write};

use crate::{ADBDeviceExt, ADBServerDevice, Result};

struct LogFilter<W: Write> {
    writer: W,
    buffer: Vec<u8>,
}

impl<W: Write> LogFilter<W> {
    pub fn new(writer: W) -> Self {
        LogFilter {
            writer,
            buffer: Vec::new(),
        }
    }

    fn should_write(&self, _line: &[u8]) -> bool {
        // Can implement checks here to ensure if logs have to be written
        true
    }
}

impl<W: Write> Write for LogFilter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);

        let buf_clone = self.buffer.clone();
        let mut lines = buf_clone.split_inclusive(|&byte| byte == b'\n').peekable();

        while let Some(line) = lines.next() {
            if lines.peek().is_some() {
                if self.should_write(line) {
                    self.writer.write_all(line)?;
                }
            } else {
                // This is the last (unfinished) element, we keep it for next round
                self.buffer = line.to_vec();
                break;
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl ADBServerDevice {
    /// Get logs from device
    pub fn get_logs<W: Write>(&mut self, output: W) -> Result<()> {
        self.shell_command(&["exec logcat"], &mut LogFilter::new(output))
    }
}
