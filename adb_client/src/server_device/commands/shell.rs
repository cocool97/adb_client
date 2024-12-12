use std::io::{Read, Write};

use crate::{
    constants::BUFFER_SIZE,
    models::{AdbServerCommand, HostFeatures},
    ADBServerDevice, Result, RustADBError,
};

impl ADBServerDevice {
    pub(crate) fn shell_command(&mut self, command: &[&str], output: &mut dyn Write) -> Result<()> {
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
            .send_adb_request(AdbServerCommand::ShellCommand(command.join(" ")))?;

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

    pub(crate) fn shell(
        &mut self,
        mut reader: &mut dyn Read,
        mut writer: Box<(dyn Write + Send)>,
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
}
