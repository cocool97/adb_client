use std::io::{Read, Write};

use crate::Result;

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
}
