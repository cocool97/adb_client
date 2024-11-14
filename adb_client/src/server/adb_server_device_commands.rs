use std::{
    io::{Read, Write},
    path::Path,
};

use crate::{
    constants::BUFFER_SIZE,
    models::{AdbServerCommand, AdbStatResponse, HostFeatures},
    ADBDeviceExt, ADBServerDevice, Result, RustADBError,
};

impl ADBDeviceExt for ADBServerDevice {
    fn shell_command<S: ToString, W: Write>(
        &mut self,
        command: impl IntoIterator<Item = S>,
        mut output: W,
    ) -> Result<()> {
        let supported_features = self.host_features()?;
        if !supported_features.contains(&HostFeatures::ShellV2)
            && !supported_features.contains(&HostFeatures::Cmd)
        {
            return Err(RustADBError::ADBShellNotSupported);
        }

        let serial = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbServerCommand::TransportSerial(serial))?;
        self.get_transport_mut()
            .send_adb_request(AdbServerCommand::ShellCommand(
                command
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
            ))?;

        const BUFFER_SIZE: usize = 4096;
        loop {
            let mut buffer = [0; BUFFER_SIZE];
            match self
                .get_transport_mut()
                .get_raw_connection()?
                .read(&mut buffer)
            {
                Ok(size) => {
                    if size == 0 {
                        return Ok(());
                    } else {
                        output.write_all(&buffer[..size])?;
                    }
                }
                Err(e) => {
                    return Err(RustADBError::IOError(e));
                }
            }
        }
    }

    fn stat(&mut self, remote_path: &str) -> Result<AdbStatResponse> {
        self.stat(remote_path)
    }

    fn shell<R: Read, W: Write + Send + 'static>(
        &mut self,
        mut reader: R,
        mut writer: W,
    ) -> Result<()> {
        let supported_features = self.host_features()?;
        if !supported_features.contains(&HostFeatures::ShellV2)
            && !supported_features.contains(&HostFeatures::Cmd)
        {
            return Err(RustADBError::ADBShellNotSupported);
        }

        let serial = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbServerCommand::TransportSerial(serial))?;
        self.get_transport_mut()
            .send_adb_request(AdbServerCommand::Shell)?;

        let mut read_stream = self.get_transport_mut().get_raw_connection()?.try_clone()?;

        let mut write_stream = read_stream.try_clone()?;

        // Reading thread, reads response from adb-server
        std::thread::spawn(move || -> Result<()> {
            loop {
                let mut buffer = [0; BUFFER_SIZE];
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
                std::io::ErrorKind::BrokenPipe => return Ok(()),
                _ => return Err(RustADBError::IOError(e)),
            }
        }

        Ok(())
    }

    fn pull<A: AsRef<str>, W: Write>(&mut self, source: A, mut output: W) -> Result<()> {
        self.pull(source, &mut output)
    }

    fn reboot(&mut self, reboot_type: crate::RebootType) -> Result<()> {
        self.reboot(reboot_type)
    }

    fn push<R: Read, A: AsRef<str>>(&mut self, stream: R, path: A) -> Result<()> {
        self.push(stream, path)
    }

    fn install<P: AsRef<Path>>(&mut self, apk_path: P) -> Result<()> {
        self.install(apk_path)
    }
}
