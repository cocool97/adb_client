use std::io::{Read, Write};

use crate::{
    models::{AdbServerCommand, HostFeatures},
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
}
