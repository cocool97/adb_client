use std::io::{Read, Write};
use std::path::Path;

use crate::models::AdbStatResponse;
use crate::{RebootType, Result};

/// Trait representing all features available on both [`crate::ADBServerDevice`] and [`crate::ADBUSBDevice`]
pub trait ADBDeviceExt {
    /// Runs command in a shell on the device, and write its output and error streams into output.
    fn shell_command<S: ToString, W: Write>(
        &mut self,
        command: impl IntoIterator<Item = S>,
        output: W,
    ) -> Result<()>;

    /// Starts an interactive shell session on the device.
    /// Input data is read from reader and write to writer.
    /// W has a 'static bound as it is internally used in a thread.
    fn shell<R: Read, W: Write + Send + 'static>(&mut self, reader: R, writer: W) -> Result<()>;

    /// Display the stat information for a remote file
    fn stat(&mut self, remote_path: &str) -> Result<AdbStatResponse>;

    /// Pull the remote file pointed to by `source` and write its contents into `output`
    fn pull<A: AsRef<str>, W: Write>(&mut self, source: A, output: W) -> Result<()>;

    /// Push `stream` to `path` on the device.
    fn push<R: Read, A: AsRef<str>>(&mut self, stream: R, path: A) -> Result<()>;

    /// Reboot the device using given reboot type
    fn reboot(&mut self, reboot_type: RebootType) -> Result<()>;

    /// Run `activity` from `package` on device. Return the command output.
    fn run_activity(&mut self, package: &str, activity: &str) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        self.shell_command(
            ["am", "start", &format!("{package}/{package}.{activity}")],
            &mut output,
        )?;

        Ok(output)
    }

    /// Install an APK pointed to by `apk_path` on device.
    fn install<P: AsRef<Path>>(&mut self, apk_path: P) -> Result<()>;
}
