#[cfg(unix)]
use std::io::{ErrorKind, Write};
use std::io::{Read};

#[cfg(unix)]
use crate::adb_termios::ADBTermios;
use crate::{
    models::{AdbCommand, HostFeatures},
    AdbTcpConnection, Result, RustADBError,
};

impl AdbTcpConnection {
    /// Runs 'command' in a shell on the device, and return its output and error streams.
    pub fn shell_command<S: ToString>(
        &mut self,
        serial: &Option<S>,
        command: impl IntoIterator<Item = S>,
    ) -> Result<Vec<u8>> {
        let supported_features = self.host_features(serial)?;
        if !supported_features.contains(&HostFeatures::ShellV2)
            && !supported_features.contains(&HostFeatures::Cmd)
        {
            return Err(RustADBError::ADBShellNotSupported);
        }

        self.new_connection()?;

        match serial {
            None => self.send_adb_request(AdbCommand::TransportAny)?,
            Some(serial) => {
                self.send_adb_request(AdbCommand::TransportSerial(serial.to_string()))?
            }
        }
        self.send_adb_request(AdbCommand::ShellCommand(
            command
                .into_iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        ))?;

        const BUFFER_SIZE: usize = 512;
        let result = (|| {
            let mut result = Vec::new();
            loop {
                let mut buffer = [0; BUFFER_SIZE];
                match self.tcp_stream.read(&mut buffer) {
                    Ok(size) => {
                        if size == 0 {
                            return Ok(result);
                        } else {
                            result.extend_from_slice(&buffer[..size]);
                        }
                    }
                    Err(e) => {
                        return Err(RustADBError::IOError(e));
                    }
                }
            }
        })();

        self.new_connection()?;
        result
    }

    /// Starts an interactive shell session on the device. Redirects stdin/stdout/stderr as appropriate.
    #[cfg(unix)]
    pub fn shell<S: ToString>(&mut self, serial: &Option<S>) -> Result<()> {
        let mut adb_termios = ADBTermios::new(std::io::stdin())?;
        adb_termios.set_adb_termios()?;

        self.tcp_stream.set_nodelay(true)?;

        // FORWARD CTRL+C !!

        let supported_features = self.host_features(serial)?;
        if !supported_features.contains(&HostFeatures::ShellV2)
            && !supported_features.contains(&HostFeatures::Cmd)
        {
            return Err(RustADBError::ADBShellNotSupported);
        }

        self.new_connection()?;

        match serial {
            None => self.send_adb_request(AdbCommand::TransportAny)?,
            Some(serial) => {
                self.send_adb_request(AdbCommand::TransportSerial(serial.to_string()))?
            }
        }
        self.send_adb_request(AdbCommand::Shell)?;

        // let read_stream = Arc::new(self.tcp_stream);
        let mut read_stream = self.tcp_stream.try_clone()?;

        // Writing thread
        let mut write_stream = read_stream.try_clone()?;
        let writer_t = std::thread::spawn(move || -> Result<()> {
            let mut buf = [0; 1024];
            loop {
                let size = std::io::stdin().read(&mut buf)?;

                write_stream.write_all(&buf[0..size])?;
            }
        });

        // Reading thread
        let reader_t = std::thread::spawn(move || -> Result<()> {
            const BUFFER_SIZE: usize = 512;
            loop {
                let mut buffer = [0; BUFFER_SIZE];
                match read_stream.read(&mut buffer) {
                    Ok(0) => {
                        return Ok(());
                    }
                    Ok(size) => {
                        std::io::stdout().write_all(&buffer[..size])?;
                        std::io::stdout().flush()?;
                    }
                    Err(e) => {
                        return Err(RustADBError::IOError(e));
                    }
                }
            }
        });

        if let Err(e) = reader_t.join().unwrap() {
            match e {
                RustADBError::IOError(e) if e.kind() == ErrorKind::BrokenPipe => {}
                _ => {
                    return Err(e);
                }
            }
        }

        if let Err(e) = writer_t.join().unwrap() {
            match e {
                RustADBError::IOError(e) if e.kind() == ErrorKind::BrokenPipe => {}
                _ => {
                    return Err(e);
                }
            }
        }

        Ok(())
    }
}
