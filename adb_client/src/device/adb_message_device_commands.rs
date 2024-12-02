use crate::{models::AdbStatResponse, ADBDeviceExt, ADBMessageTransport, RebootType, Result};
use std::io::{Read, Write};

use super::ADBMessageDevice;

impl<T: ADBMessageTransport> ADBDeviceExt for ADBMessageDevice<T> {
    fn shell_command<S: ToString, W: Write>(
        &mut self,
        command: impl IntoIterator<Item = S>,
        output: W,
    ) -> Result<()> {
        self.shell_command(command, output)
    }

    fn shell<R: Read, W: Write + Send + 'static>(&mut self, reader: R, writer: W) -> Result<()> {
        self.shell(reader, writer)
    }

    fn stat(&mut self, remote_path: &str) -> Result<AdbStatResponse> {
        self.stat(remote_path)
    }

    fn pull<A: AsRef<str>, W: Write>(&mut self, source: A, output: W) -> Result<()> {
        self.pull(source, output)
    }

    fn push<R: Read, A: AsRef<str>>(&mut self, stream: R, path: A) -> Result<()> {
        self.push(stream, path)
    }

    fn reboot(&mut self, reboot_type: RebootType) -> Result<()> {
        self.reboot(reboot_type)
    }

    fn install<P: AsRef<std::path::Path>>(&mut self, apk_path: P) -> Result<()> {
        self.install(apk_path)
    }

    fn framebuffer<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        self.framebuffer(path)
    }

    fn framebuffer_bytes<W: Write + std::io::Seek>(&mut self, writer: W) -> Result<()> {
        self.framebuffer_bytes(writer)
    }
}
