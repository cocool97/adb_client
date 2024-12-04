use std::io::{Read, Seek, Write};
use std::path::Path;

use image::{ImageBuffer, ImageFormat, Rgba};

use crate::models::AdbStatResponse;
use crate::{RebootType, Result};

/// Trait representing all features available on both [`crate::ADBServerDevice`] and [`crate::ADBUSBDevice`]
pub trait ADBDeviceExt {
    /// Runs command in a shell on the device, and write its output and error streams into output.
    fn shell_command<S: ToString, W: Write>(
        &mut self,
        command: impl IntoIterator<Item = S>,
        output: W,
    ) -> Result<()>
    where
        Self: Sized;

    /// Starts an interactive shell session on the device.
    /// Input data is read from reader and write to writer.
    /// W has a 'static bound as it is internally used in a thread.
    fn shell<R: Read, W: Write + Send + 'static>(&mut self, reader: R, writer: W) -> Result<()>
    where
        Self: Sized;

    /// Display the stat information for a remote file
    fn stat(&mut self, remote_path: &str) -> Result<AdbStatResponse>;

    /// Pull the remote file pointed to by `source` and write its contents into `output`
    fn pull<A: AsRef<str>, W: Write>(&mut self, source: A, output: W) -> Result<()>
    where
        Self: Sized;

    /// Push `stream` to `path` on the device.
    fn push<R: Read, A: AsRef<str>>(&mut self, stream: R, path: A) -> Result<()>
    where
        Self: Sized;

    /// Reboot the device using given reboot type
    fn reboot(&mut self, reboot_type: RebootType) -> Result<()>;

    /// Run `activity` from `package` on device. Return the command output.
    fn run_activity(&mut self, package: &str, activity: &str) -> Result<Vec<u8>>
    where
        Self: Sized,
    {
        let mut output = Vec::new();
        self.shell_command(
            ["am", "start", &format!("{package}/{package}.{activity}")],
            &mut output,
        )?;

        Ok(output)
    }

    /// Install an APK pointed to by `apk_path` on device.
    fn install<P: AsRef<Path>>(&mut self, apk_path: P) -> Result<()>
    where
        Self: Sized;

    /// Inner method requesting framebuffer from an Android device
    fn framebuffer_inner(&mut self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>>;

    /// Dump framebuffer of this device into given path
    fn framebuffer<P: AsRef<Path>>(&mut self, path: P) -> Result<()>
    where
        Self: Sized,
    {
        // Big help from AOSP source code (<https://android.googlesource.com/platform/system/adb/+/refs/heads/main/framebuffer_service.cpp>)
        let img = self.framebuffer_inner()?;
        Ok(img.save(path.as_ref())?)
    }

    /// Dump framebuffer of this device and return corresponding bytes.
    ///
    /// Output data format is currently only `PNG`.
    fn framebuffer_bytes<W: Write + Seek>(&mut self, mut writer: W) -> Result<()>
    where
        Self: Sized,
    {
        let img = self.framebuffer_inner()?;
        Ok(img.write_to(&mut writer, ImageFormat::Png)?)
    }
}
