use std::{
    io::{self, Read, Write},
    sync::mpsc,
    time::Duration,
};

use mio::{unix::SourceFd, Events, Interest, Poll, Token};

use crate::{
    adb_termios::ADBTermios,
    models::{AdbServerCommand, HostFeatures},
    ADBServerDevice, Result, RustADBError,
};

const STDIN: Token = Token(0);
const BUFFER_SIZE: usize = 512;
const POLL_DURATION: Duration = Duration::from_millis(100);

fn setup_poll_stdin() -> std::result::Result<Poll, io::Error> {
    let poll = Poll::new()?;
    let stdin_fd = 0;
    poll.registry()
        .register(&mut SourceFd(&stdin_fd), STDIN, Interest::READABLE)?;

    Ok(poll)
}

impl ADBServerDevice {
    /// Runs 'command' in a shell on the device, and write its output and error streams into [`output`].
    pub fn shell_command<S: ToString, W: Write>(
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

    /// Starts an interactive shell session on the device. Redirects stdin/stdout/stderr as appropriate.
    pub fn shell(&mut self) -> Result<()> {
        let mut adb_termios = ADBTermios::new(std::io::stdin())?;
        adb_termios.set_adb_termios()?;

        // TODO: FORWARD CTRL+C !!

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

        // let read_stream = Arc::new(self.tcp_stream);
        let mut read_stream = self.get_transport_mut().get_raw_connection()?.try_clone()?;

        let (tx, rx) = mpsc::channel::<bool>();

        let mut write_stream = read_stream.try_clone()?;

        // Reading thread
        std::thread::spawn(move || -> Result<()> {
            loop {
                let mut buffer = [0; BUFFER_SIZE];
                match read_stream.read(&mut buffer) {
                    Ok(0) => {
                        let _ = tx.send(true);
                        read_stream.shutdown(std::net::Shutdown::Both)?;
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

        let mut buf = [0; BUFFER_SIZE];
        let mut events = Events::with_capacity(1);

        let mut poll = setup_poll_stdin()?;

        // Polling either by checking that reading socket hasn't been closed, and if is there is something to read on stdin.
        loop {
            poll.poll(&mut events, Some(POLL_DURATION))?;
            match rx.try_recv() {
                Ok(_) | Err(mpsc::TryRecvError::Disconnected) => return Ok(()),
                Err(mpsc::TryRecvError::Empty) => (),
            }

            for event in events.iter() {
                match event.token() {
                    STDIN => {
                        let size = match std::io::stdin().read(&mut buf) {
                            Ok(0) => return Ok(()),
                            Ok(size) => size,
                            Err(_) => return Ok(()),
                        };

                        write_stream.write_all(&buf[0..size])?;
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}
