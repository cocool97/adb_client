use std::io::{Read, Write};

use crate::{RebootType, Result};
use serde::{Deserialize, Serialize};

/// Outputs of the `STAT` command on a remote file
#[derive(Debug, Serialize, Deserialize)]
pub struct FileStat {
    /// mode of the file; 0 represents unavailable
    pub mode: u32,
    /// size of the file if it exists
    pub file_size: u32,
}

/// Trait representing all features available on both [`ADBServerDevice`] and [`ADBUSBDevice`]
pub trait ADBDeviceExt {
    /// Runs 'command' in a shell on the device, and write its output and error streams into [`output`].
    fn shell_command<S: ToString, W: Write>(
        &mut self,
        command: impl IntoIterator<Item = S>,
        output: W,
    ) -> Result<()>;

    /// Starts an interactive shell session on the device.
    /// Input data is read from [reader] and write to [writer].
    /// [W] has a 'static bound as it is internally used in a thread.
    fn shell<R: Read, W: Write + Send + 'static>(&mut self, reader: R, writer: W) -> Result<()>;

    /// Display the stat for a remote file
    fn stat(&mut self, remote_path: &str, local_id: u32, remote_id: u32) -> Result<FileStat>;

    /// Pull the remote file `source` and write its contents into [`output`]
    fn pull<A: AsRef<str>, W: Write>(&mut self, source: A, output: W) -> Result<()>;

    /// Reboots the device using given reboot type
    fn reboot(&mut self, reboot_type: RebootType) -> Result<()>;
}
