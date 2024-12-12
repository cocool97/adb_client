use std::{
    io::{Read, Write},
    path::Path,
};

use crate::{models::AdbStatResponse, ADBDeviceExt, ADBProtoPort, Result};

use super::ADBServerDevice;

impl ADBDeviceExt for ADBServerDevice {
    fn shell_command(&mut self, command: &[&str], output: &mut dyn Write) -> Result<()> {
        self.shell_command(command, output)
    }

    fn stat(&mut self, remote_path: &str) -> Result<AdbStatResponse> {
        self.stat(remote_path)
    }

    fn shell(&mut self, reader: &mut dyn Read, writer: Box<(dyn Write + Send)>) -> Result<()> {
        self.shell(reader, writer)
    }

    fn pull(&mut self, source: &dyn AsRef<str>, mut output: &mut dyn Write) -> Result<()> {
        self.pull(source, &mut output)
    }

    fn reboot(&mut self, reboot_type: crate::RebootType) -> Result<()> {
        self.reboot(reboot_type)
    }

    fn push(&mut self, stream: &mut dyn Read, path: &dyn AsRef<str>) -> Result<()> {
        self.push(stream, path)
    }

    fn install(&mut self, apk_path: &dyn AsRef<Path>) -> Result<()> {
        self.install(apk_path)
    }

    fn framebuffer_inner(&mut self) -> Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>> {
        self.framebuffer_inner()
    }

    fn forward(&mut self, remote: ADBProtoPort, local: ADBProtoPort) -> Result<()> {
        self.forward(remote, local)
    }

    fn forward_remove_all(&mut self) -> Result<()> {
        self.forward_remove_all()
    }

    fn reverse(&mut self, remote: ADBProtoPort, local: ADBProtoPort) -> Result<()> {
        self.reverse(remote, local)
    }

    fn reverse_remove_all(&mut self) -> Result<()> {
        self.reverse_remove_all()
    }
}
