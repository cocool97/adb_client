use std::io::{Cursor, Read, Write};
use std::path::Path;

use image::{ImageBuffer, ImageFormat, Rgba};

use crate::models::{ADBListItemType, AdbStatResponse, RemountInfo};
use crate::{ADBStatExtendedResponse, RebootType, Result};

/// Trait representing all features available on ADB devices.
pub trait ADBDeviceExt {
    /// Runs command in a shell on the device, and write its output and error streams into output.
    fn shell_command<W: Write>(
        &mut self,
        command: &str,
        stdout: Option<&mut W>,
        stderr: Option<&mut W>,
    ) -> Result<Option<u8>>;

    /// Starts an interactive shell session on the device.
    /// Input data is read from reader and write to writer.
    fn shell<R: Read, W: Write + Send>(&mut self, reader: &mut R, writer: W) -> Result<()>;

    /// Runs command on the device.
    /// Input data is read from reader and write to writer.
    fn exec<R: Read, W: Write + Send>(
        &mut self,
        command: &str,
        reader: &mut R,
        writer: W,
    ) -> Result<()>;

    /// Display the stat information for a remote file using STAT protocol command.
    fn stat<P: AsRef<Path>>(&mut self, remote_path: P) -> Result<AdbStatResponse>;

    /// Display the stat information for a remote file using `stat` shell command.
    /// This is an extended version of `stat` that returns more detailed information.
    /// Returns `Ok(None)` if the file does not exist on the device.
    fn stat_extended<P: AsRef<Path>>(
        &mut self,
        remote_path: P,
    ) -> Result<Option<ADBStatExtendedResponse>> {
        let mut stdout = Vec::new();
        self.shell_command(
            &format!("stat {}", remote_path.as_ref().display()),
            Some(&mut stdout),
            None,
        )?;

        // all parsing magic happens here...
        ADBStatExtendedResponse::try_from(&stdout)
    }

    /// Pull the remote file pointed to by `source` and write its contents into `output`
    fn pull<P: AsRef<Path>, W: Write>(&mut self, source: P, output: &mut W) -> Result<()>;

    /// Push `stream` to `path` on the device.
    fn push<R: Read, P: AsRef<Path>>(&mut self, stream: &mut R, path: P) -> Result<()>;

    /// List the items in a directory on the device
    fn list<P: AsRef<Path>>(&mut self, path: P) -> Result<Vec<ADBListItemType>>;

    /// Reboot the device using given reboot type
    fn reboot(&mut self, reboot_type: RebootType) -> Result<()>;

    /// Remount the device partitions as read-write
    fn remount(&mut self) -> Result<Vec<RemountInfo>>;

    /// Restart adb daemon with root permissions
    fn root(&mut self) -> Result<()>;

    /// Run `activity` from `package` on device. Return the command output.
    fn run_activity(&mut self, package: &str, activity: &str) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        let _status = self.shell_command(
            &format!("am start {package}/{package}.{activity}"),
            Some(&mut output),
            None,
        )?;

        Ok(output)
    }

    /// Install an APK pointed to by `apk_path` on device.
    fn install<P: AsRef<Path>>(&mut self, apk_path: P, user: Option<&str>) -> Result<()>;

    /// Uninstall the package `package` from device.
    fn uninstall(&mut self, package: &str, user: Option<&str>) -> Result<()>;

    /// Enable dm-verity on the device
    fn enable_verity(&mut self) -> Result<()>;

    /// Disable dm-verity on the device
    fn disable_verity(&mut self) -> Result<()>;

    /// Inner method requesting framebuffer from an Android device
    fn framebuffer_inner(&mut self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>>;

    /// Dump framebuffer of this device into given path.
    ///
    /// Output data format is currently only `PNG`.
    fn framebuffer<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
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
}
