use crate::{
    ADBDeviceExt, ADBListItemType, RebootType, Result,
    message_devices::{
        adb_message_device::ADBMessageDevice, adb_message_transport::ADBMessageTransport,
    },
    models::{AdbStatResponse, RemountInfo},
};
use std::{
    io::{Read, Write},
    path::Path,
};

impl<T: ADBMessageTransport> ADBDeviceExt for ADBMessageDevice<T> {
    #[inline]
    fn shell_command<W: Write>(
        &mut self,
        command: &str,
        stdout: Option<&mut W>,
        stderr: Option<&mut W>,
    ) -> Result<Option<u8>> {
        self.shell_command(command, stdout, stderr)
    }

    #[inline]
    fn shell<R: Read, W: Write + Send>(&mut self, reader: &mut R, writer: W) -> Result<()> {
        self.shell(reader, writer)
    }

    #[inline]
    fn exec<R: Read, W: Write + Send>(
        &mut self,
        command: &str,
        reader: &mut R,
        writer: W,
    ) -> Result<()> {
        self.exec(command, reader, writer)
    }

    #[inline]
    fn stat<P: AsRef<Path>>(&mut self, remote_path: P) -> Result<AdbStatResponse> {
        self.stat(remote_path)
    }

    #[inline]
    fn pull<P: AsRef<Path>, W: Write>(&mut self, source: P, output: &mut W) -> Result<()> {
        self.pull(source, output)
    }

    #[inline]
    fn push<R: Read, P: AsRef<Path>>(&mut self, stream: &mut R, path: P) -> Result<()> {
        self.push(stream, path)
    }

    #[inline]
    fn reboot(&mut self, reboot_type: RebootType) -> Result<()> {
        self.reboot(reboot_type)
    }

    #[inline]
    fn remount(&mut self) -> Result<Vec<RemountInfo>> {
        self.remount()
    }

    #[inline]
    fn root(&mut self) -> Result<()> {
        self.root()
    }

    #[inline]
    fn install<P: AsRef<Path>>(&mut self, apk_path: P, user: Option<&str>) -> Result<()> {
        self.install(apk_path, user)
    }

    #[inline]
    fn uninstall(&mut self, package: &str, user: Option<&str>) -> Result<()> {
        self.uninstall(package, user)
    }

    #[inline]
    fn enable_verity(&mut self) -> Result<()> {
        self.enable_verity()
    }

    #[inline]
    fn disable_verity(&mut self) -> Result<()> {
        self.disable_verity()
    }

    #[inline]
    fn framebuffer_inner(&mut self) -> Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>> {
        self.framebuffer_inner()
    }

    #[inline]
    fn list<P: AsRef<Path>>(&mut self, path: P) -> Result<Vec<ADBListItemType>> {
        self.list(path)
    }
}
