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

        let mut lines = buf_clone.split_inclusive(|&byte| byte == b'\n');
        let mut offset = 0;
        for line in lines {
            let is_line = self.buffer.last().unwrap() == &b'\n';
            if is_line {
                offset += line.len();
                if self.should_write(line) {
                    self.writer.write_all(line)?;
                }
            }
        }

        self.buffer.as_mut_slice().copy_within(offset.., 0);
        self.buffer.truncate(self.buffer[offset..].len());

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
