use std::io::{Cursor, Read, Write};
use std::path::Path;

use image::{ImageBuffer, ImageFormat, Rgba};

use crate::models::AdbStatResponse;
use crate::{RebootType, Result};

/// Trait representing all features available on both [`crate::ADBServerDevice`] and [`crate::ADBUSBDevice`]
pub trait ADBDeviceExt {
    /// Runs command in a shell on the device, and write its output and error streams into output.
    fn shell_command(&mut self, command: &[&str], output: &mut dyn Write) -> Result<()>;

    /// Starts an interactive shell session on the device.
    /// Input data is read from reader and write to writer.
    fn shell(&mut self, reader: &mut dyn Read, writer: Box<(dyn Write + Send)>) -> Result<()>;

    /// Display the stat information for a remote file
    fn stat(&mut self, remote_path: &str) -> Result<AdbStatResponse>;

    /// Pull the remote file pointed to by `source` and write its contents into `output`
    fn pull(&mut self, source: &dyn AsRef<str>, output: &mut dyn Write) -> Result<()>;

    /// Push `stream` to `path` on the device.
    fn push(&mut self, stream: &mut dyn Read, path: &dyn AsRef<str>) -> Result<()>;

    /// Reboot the device using given reboot type
    fn reboot(&mut self, reboot_type: RebootType) -> Result<()>;

    /// Run `activity` from `package` on device. Return the command output.
    fn run_activity(&mut self, package: &str, activity: &str) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        self.shell_command(
            &["am", "start", &format!("{package}/{package}.{activity}")],
            &mut output,
        )?;

        Ok(output)
    }

    /// Install an APK pointed to by `apk_path` on device.
    fn install(&mut self, apk_path: &dyn AsRef<Path>) -> Result<()>;

    /// Uninstall the package `package` from device.
    fn uninstall(&mut self, package: &str) -> Result<()>;

    /// Inner method requesting framebuffer from an Android device
    fn framebuffer_inner(&mut self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>>;

    /// Dump framebuffer of this device into given path
    fn framebuffer(&mut self, path: &dyn AsRef<Path>) -> Result<()> {
        // Big help from AOSP source code (<https://android.googlesource.com/platform/system/adb/+/refs/heads/main/framebuffer_service.cpp>)
        let img = self.framebuffer_inner()?;
        Ok(img.save(path.as_ref())?)
    }

    /// Dump framebuffer of this device and return corresponding bytes.
    ///
    /// Output data format is currently only `PNG`.
    fn framebuffer_bytes(&mut self) -> Result<Vec<u8>> {
        let img = self.framebuffer_inner()?;
        let mut vec = Cursor::new(Vec::new());
        img.write_to(&mut vec, ImageFormat::Png)?;

        Ok(vec.into_inner())
    }

    /// Return a boxed instance representing this trait
    fn boxed(self) -> Box<dyn ADBDeviceExt>
    where
        Self: Sized,
        Self: 'static,
    {
        Box::new(self)
    }
}
