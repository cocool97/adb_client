use std::io::Write;

use crate::Result;

/// Trait representing all features available on both [`ADBServerDevice`] and [`ADBUSBDevice`]
pub trait ADBDeviceExt {
    /// Runs 'command' in a shell on the device, and write its output and error streams into [`output`].
    fn shell_command<S: ToString, W: Write>(
        &mut self,
        command: impl IntoIterator<Item = S>,
        output: W,
    ) -> Result<()>;
}
