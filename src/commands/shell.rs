use std::io::{ErrorKind, Read, Write};

use crate::{
    adb_termios::ADBTermios,
    models::{AdbCommand, HostFeatures},
    AdbTcpConnexion, Result, RustADBError,
};

impl AdbTcpConnexion {
    /// Runs 'command' in a shell on the device, and return its output and error streams.
    pub fn shell_command<S: ToString>(
        &mut self,
        serial: &Option<S>,
        command: impl IntoIterator<Item = S>,
    ) -> Result<()> {
        let supported_features = self.host_features(serial)?;
        if !supported_features.contains(&HostFeatures::ShellV2)
            && !supported_features.contains(&HostFeatures::Cmd)
        {
            return Err(RustADBError::ADBShellNotSupported);
        }

        self.new_connection()?;

        match serial {
            None => Self::send_adb_request(&mut self.tcp_stream, AdbCommand::TransportAny)?,
            Some(serial) => Self::send_adb_request(
                &mut self.tcp_stream,
                AdbCommand::TransportSerial(serial.to_string()),
            )?,
        }
        Self::send_adb_request(
            &mut self.tcp_stream,
            AdbCommand::ShellCommand(
                command
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
        )?;

        let buffer_size = 512;
        loop {
            let mut buffer = vec![0; buffer_size];
            match self.tcp_stream.read(&mut buffer) {
                Ok(size) => {
                    if size == 0 {
                        return Ok(());
                    } else {
                        print!("{}", String::from_utf8(buffer.to_vec())?);
                        std::io::stdout().flush()?;
                    }
                }
                Err(e) => {
                    return Err(RustADBError::IOError(e));
                }
            }
        }
    }

    /// Starts an interactive shell session on the device. Redirects stdin/stdout/stderr as appropriate.
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
            None => Self::send_adb_request(&mut self.tcp_stream, AdbCommand::TransportAny)?,
            Some(serial) => Self::send_adb_request(
                &mut self.tcp_stream,
                AdbCommand::TransportSerial(serial.to_string()),
            )?,
        }
        Self::send_adb_request(&mut self.tcp_stream, AdbCommand::Shell)?;

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
            let buffer_size = 512;
            loop {
                let mut buffer = vec![0; buffer_size];
                match read_stream.read(&mut buffer) {
                    Ok(size) if size == 0 => {
                        // TODO: check if return here is good.. return Ok(()) ?

                        // return Err(RustADBError::IOError(std::io::Error::from(
                        //     ErrorKind::BrokenPipe,
                        // )));

                        return Ok(());
                    }
                    Ok(_) => {
                        print!("{}", String::from_utf8(buffer.to_vec())?);
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
