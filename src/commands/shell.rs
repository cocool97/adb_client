use std::{
    io::{ErrorKind, Read, Write},
    net::TcpStream,
    sync::Arc,
};

use crate::{adb_termios::ADBTermios, models::AdbCommand, AdbTcpConnexion, Result, RustADBError};

impl AdbTcpConnexion {
    /// Runs 'command' in a shell on the device, and return its output and error streams.
    pub fn shell_command<S: ToString>(&self, serial: Option<S>, command: Vec<S>) -> Result<()> {
        let mut tcp_stream = TcpStream::connect(self.socket_addr)?;

        // TODO: Add host:features

        match serial {
            None => Self::send_adb_request(&mut tcp_stream, AdbCommand::TransportAny)?,
            Some(serial) => Self::send_adb_request(
                &mut tcp_stream,
                AdbCommand::TransportSerial(serial.to_string()),
            )?,
        }
        Self::send_adb_request(
            &mut tcp_stream,
            AdbCommand::ShellCommand(
                command
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
            ),
        )?;

        let buffer_size = 512;
        loop {
            let mut buffer = vec![0; buffer_size];
            match tcp_stream.read(&mut buffer) {
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
    pub fn shell<S: ToString>(&self, serial: Option<S>) -> Result<()> {
        let mut adb_termios = ADBTermios::new(std::io::stdin())?;
        adb_termios.set_adb_termios()?;

        let mut tcp_stream = TcpStream::connect(self.socket_addr)?;
        tcp_stream.set_nodelay(true)?;

        // TODO: Add host:features

        match serial {
            None => Self::send_adb_request(&mut tcp_stream, AdbCommand::TransportAny)?,
            Some(serial) => Self::send_adb_request(
                &mut tcp_stream,
                AdbCommand::TransportSerial(serial.to_string()),
            )?,
        }
        Self::send_adb_request(&mut tcp_stream, AdbCommand::Shell)?;

        let read_stream = Arc::new(tcp_stream);

        // TODO: Send terminal informations

        // Writing thread
        let write_stream = read_stream.clone();
        let writer_t = std::thread::spawn(move || -> Result<()> {
            let mut buf = [0; 1024];
            loop {
                let size = std::io::stdin().read(&mut buf)?;

                (&*write_stream).write_all(&buf[0..size])?;
            }
        });

        // Reading thread
        let reader_t = std::thread::spawn(move || -> Result<()> {
            let buffer_size = 512;
            loop {
                let mut buffer = vec![0; buffer_size];
                match (&*read_stream).read(&mut buffer) {
                    Ok(size) if size == 0 => {
                        return Err(RustADBError::IOError(std::io::Error::from(
                            ErrorKind::BrokenPipe,
                        )));
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
