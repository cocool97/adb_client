use std::{
    io::{ErrorKind, Read, Write},
    path::Path,
};

use crate::{
    ADBDeviceExt, ADBListItemType, Result, RustADBError,
    models::{AdbStatResponse, HostFeatures},
    server::AdbServerCommand,
};

use super::ADBServerDevice;

const BUFFER_SIZE: usize = 65535;

impl ADBDeviceExt for ADBServerDevice {
    fn shell_command(&mut self, command: &[&str], output: &mut dyn Write) -> Result<()> {
        let supported_features = self.host_features()?;
        if !supported_features.contains(&HostFeatures::ShellV2)
            && !supported_features.contains(&HostFeatures::Cmd)
        {
            return Err(RustADBError::ADBShellNotSupported);
        }

        self.set_serial_transport()?;

        // Prepare shell command arguments
        let mut args = Vec::new();
        let command_string = command.join(" ");

        // Add v2 mode if supported
        if supported_features.contains(&HostFeatures::ShellV2) {
            log::debug!("using shell_v2 feature");
            args.push("v2".to_string());
        }

        // Include terminal information if available
        if let Ok(term) = std::env::var("TERM") {
            args.push(format!("TERM={term}"));
        }

        // Send the request
        self.transport
            .send_adb_request(&AdbServerCommand::ShellCommand(command_string, args))?;

        let mut buffer = vec![0; BUFFER_SIZE].into_boxed_slice();
        loop {
            match self.transport.get_raw_connection()?.read(&mut buffer) {
                Ok(size) => {
                    if size == 0 {
                        return Ok(());
                    }
                    output.write_all(&buffer[..size])?;
                }
                Err(e) => {
                    return Err(RustADBError::IOError(e));
                }
            }
        }
    }

    #[inline]
    fn stat(&mut self, remote_path: &dyn AsRef<str>) -> Result<AdbStatResponse> {
        self.stat(remote_path.as_ref())
    }

    fn shell(
        &mut self,
        mut reader: &mut dyn Read,
        mut writer: Box<dyn Write + Send>,
    ) -> Result<()> {
        let supported_features = self.host_features()?;
        if !supported_features.contains(&HostFeatures::ShellV2)
            && !supported_features.contains(&HostFeatures::Cmd)
        {
            return Err(RustADBError::ADBShellNotSupported);
        }

        self.set_serial_transport()?;
        self.transport.send_adb_request(&AdbServerCommand::Shell)?;

        let mut read_stream = self.transport.get_raw_connection()?.try_clone()?;

        let mut write_stream = read_stream.try_clone()?;

        // Reading thread, reads response from adb-server
        std::thread::spawn(move || -> Result<()> {
            let mut buffer = vec![0; BUFFER_SIZE].into_boxed_slice();

            loop {
                match read_stream.read(&mut buffer) {
                    Ok(0) => {
                        read_stream.shutdown(std::net::Shutdown::Both)?;
                        return Ok(());
                    }
                    Ok(size) => {
                        writer.write_all(&buffer[..size])?;
                        writer.flush()?;
                    }
                    Err(e) => {
                        return Err(RustADBError::IOError(e));
                    }
                }
            }
        });

        // Read from given reader (that could be stdin e.g), and write content to server socket
        if let Err(e) = std::io::copy(&mut reader, &mut write_stream) {
            match e.kind() {
                ErrorKind::BrokenPipe => return Ok(()),
                _ => return Err(RustADBError::IOError(e)),
            }
        }

        Ok(())
    }

    #[inline]
    fn pull(&mut self, source: &dyn AsRef<str>, mut output: &mut dyn Write) -> Result<()> {
        self.pull(source, &mut output)
    }

    #[inline]
    fn reboot(&mut self, reboot_type: crate::RebootType) -> Result<()> {
        self.reboot(reboot_type)
    }

    #[inline]
    fn push(&mut self, stream: &mut dyn Read, path: &dyn AsRef<str>) -> Result<()> {
        self.push(stream, path)
    }

    #[inline]
    fn install(&mut self, apk_path: &dyn AsRef<Path>) -> Result<()> {
        self.install(apk_path)
    }

    #[inline]
    fn uninstall(&mut self, package: &dyn AsRef<str>) -> Result<()> {
        self.uninstall(package.as_ref())
    }

    #[inline]
    fn framebuffer_inner(&mut self) -> Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>> {
        self.framebuffer_inner()
    }

    #[inline]
    fn list(&mut self, path: &dyn AsRef<str>) -> Result<Vec<ADBListItemType>> {
        self.list(path)
    }
}
