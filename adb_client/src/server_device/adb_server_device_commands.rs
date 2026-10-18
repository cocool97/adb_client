use std::{
    io::{ErrorKind, Read, Write},
    path::Path,
};

use byteorder::ReadBytesExt;

use crate::{
    ADBDeviceExt, ADBListItemType, Result, RustADBError,
    models::{ADBCommand, ADBLocalCommand, AdbStatResponse, HostFeatures, RemountInfo},
};

use super::ADBServerDevice;

const BUFFER_SIZE: usize = 65535;

#[derive(Eq, PartialEq)]
enum ShellChannel {
    Stdout,
    Stderr,
    ExitStatus,
}

impl TryFrom<u8> for ShellChannel {
    type Error = std::io::Error;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Stdout),
            2 => Ok(Self::Stderr),
            3 => Ok(Self::ExitStatus),
            _ => Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "Invalid channel",
            )),
        }
    }
}

impl ADBDeviceExt for ADBServerDevice {
    fn shell_command(
        &mut self,
        command: &dyn AsRef<str>,
        stdout: Option<&mut dyn Write>,
        stderr: Option<&mut dyn Write>,
    ) -> Result<Option<u8>> {
        let supported_features = self.host_features();
        let use_shell_v2 = supported_features.is_ok_and(|features| {
            features.contains(&HostFeatures::ShellV2) || features.contains(&HostFeatures::Cmd)
        });

        self.set_serial_transport()?;

        if use_shell_v2 {
            self.shell_command_v2(command, stdout, stderr)
        } else {
            self.shell_command_v1(command, stdout)
        }
    }

    #[inline]
    fn stat(&mut self, remote_path: &dyn AsRef<str>) -> Result<AdbStatResponse> {
        self.stat(remote_path.as_ref())
    }

    fn exec(
        &mut self,
        command: &str,
        reader: Box<dyn Read + Send>,
        writer: Box<dyn Write + Send>,
    ) -> Result<()> {
        self.bidirectional_session(
            &ADBCommand::Local(ADBLocalCommand::Exec(command.to_owned())),
            reader,
            writer,
        )
    }

    fn shell(&mut self, reader: Box<dyn Read + Send>, writer: Box<dyn Write + Send>) -> Result<()> {
        self.bidirectional_session(&ADBCommand::Local(ADBLocalCommand::Shell), reader, writer)
    }

    fn pull(&mut self, source: &dyn AsRef<str>, mut output: &mut dyn Write) -> Result<()> {
        self.pull(source, &mut output)
    }

    fn reboot(&mut self, reboot_type: crate::RebootType) -> Result<()> {
        self.reboot(reboot_type)
    }

    fn root(&mut self) -> Result<()> {
        self.root()
    }

    fn push(&mut self, stream: &mut dyn Read, path: &dyn AsRef<str>) -> Result<()> {
        self.push(stream, path)
    }

    fn install(&mut self, apk_path: &dyn AsRef<Path>, user: Option<&str>) -> Result<()> {
        self.install(apk_path, user)
    }

    fn uninstall(&mut self, package: &dyn AsRef<str>, user: Option<&str>) -> Result<()> {
        self.uninstall(package.as_ref(), user)
    }

    #[cfg(feature = "framebuffer")]
    fn framebuffer_inner(&mut self) -> Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>> {
        self.framebuffer_inner()
    }

    fn list(&mut self, path: &dyn AsRef<str>) -> Result<Vec<ADBListItemType>> {
        self.list(path)
    }

    fn remount(&mut self) -> Result<Vec<RemountInfo>> {
        self.remount()
    }

    fn enable_verity(&mut self) -> Result<()> {
        self.enable_verity()
    }

    fn disable_verity(&mut self) -> Result<()> {
        self.disable_verity()
    }
}

impl ADBServerDevice {
    /// Shell v1: simple shell without protocol (for older ADB versions)
    fn shell_command_v1(
        &self,
        command: &dyn AsRef<str>,
        mut stdout: Option<&mut dyn Write>,
    ) -> Result<Option<u8>> {
        self.transport
            .send_adb_request(&ADBCommand::Local(ADBLocalCommand::ShellCommand(
                command.as_ref().to_string(),
                vec![],
            )))?;

        let mut input = self.transport.get_raw_connection()?;
        let mut buffer = vec![0; BUFFER_SIZE].into_boxed_slice();

        loop {
            match input.read(&mut buffer) {
                Ok(0) => break,
                Ok(size) => {
                    if let Some(stdout) = stdout.as_mut() {
                        stdout.write_all(&buffer[..size])?;
                    }
                }
                Err(e) => match e.kind() {
                    ErrorKind::UnexpectedEof | ErrorKind::BrokenPipe => break,
                    _ => return Err(RustADBError::IOError(e)),
                },
            }
        }

        Ok(None)
    }

    /// Shell v2: with protocol packets (for newer ADB versions)
    fn shell_command_v2(
        &self,
        command: &dyn AsRef<str>,
        mut stdout: Option<&mut dyn Write>,
        mut stderr: Option<&mut dyn Write>,
    ) -> Result<Option<u8>> {
        let mut args = vec!["v2".to_string()];

        if let Ok(term) = std::env::var("TERM") {
            args.push(format!("TERM={term}"));
        }

        // Send the request
        self.transport
            .send_adb_request(&ADBCommand::Local(ADBLocalCommand::ShellCommand(
                command.as_ref().to_string(),
                args,
            )))?;

        // Now decode the shell v2 protocol packets, reference:
        // https://android.googlesource.com/platform/packages/modules/adb/+/refs/heads/main/shell_protocol.h

        let mut exit = None;
        let mut input = std::io::BufReader::new(self.transport.get_raw_connection()?);

        let mut buffer = vec![0; BUFFER_SIZE].into_boxed_slice();
        loop {
            // 1 byte of channel
            // 4 bytes of payload size
            let mut pckt_metadata = vec![0; 5];
            if let Err(err) = input.read_exact(&mut pckt_metadata) {
                match err.kind() {
                    ErrorKind::UnexpectedEof | ErrorKind::BrokenPipe => return Ok(None),
                    _ => return Err(RustADBError::IOError(err)),
                }
            }

            let (channel, payload_size) = {
                let channel = pckt_metadata[0];
                let payload_size = u32::from_le_bytes(pckt_metadata[1..5].try_into()?) as usize;
                (ShellChannel::try_from(channel)?, payload_size)
            };

            if payload_size == 0 {
                continue;
            }

            match channel {
                ShellChannel::Stdout | ShellChannel::Stderr => {
                    let mut remainder = payload_size;
                    while remainder > 0 {
                        let to_read = std::cmp::min(remainder, BUFFER_SIZE);
                        match input.read(&mut buffer[0..to_read]) {
                            Ok(size) => {
                                if size == 0 {
                                    return Ok(exit);
                                }

                                match channel {
                                    ShellChannel::Stdout => {
                                        if let Some(stdout) = stdout.as_mut() {
                                            stdout.write_all(&buffer[..size])?;
                                        }
                                    }
                                    ShellChannel::Stderr => {
                                        // first stderr if existing, else a merged output into stdout
                                        if let Some(writer) = stderr.as_mut() {
                                            writer.write_all(&buffer[..size])?;
                                        } else if let Some(writer) = stdout.as_mut() {
                                            writer.write_all(&buffer[..size])?;
                                        }
                                    }
                                    ShellChannel::ExitStatus => {
                                        // unreachable
                                    }
                                }

                                remainder -= size;
                            }
                            Err(e) => {
                                return Err(RustADBError::IOError(e));
                            }
                        }
                    }
                }
                ShellChannel::ExitStatus => {
                    if payload_size != 1 {
                        return Err(RustADBError::ADBShellV2ParseError(format!(
                            "Spurious exit status packet with size of {payload_size} (should be 1)"
                        )));
                    }

                    match input.read_u8() {
                        Ok(status) => exit = Some(status),
                        Err(err) => match err.kind() {
                            ErrorKind::UnexpectedEof | ErrorKind::BrokenPipe => return Ok(None),
                            _ => return Err(RustADBError::IOError(err)),
                        },
                    }
                }
            }
        }
    }

    /// Starts a bidirectional (interactive) session. This can be a shell or an exec session.
    ///
    /// Data read from `reader` is forwarded to the server from a background thread, while server output
    /// is written to `writer` from the current thread. The session ends once the server closes the stream:
    /// either because the remote command exited, or because we half-closed it after `reader` reached EOF.
    fn bidirectional_session(
        &mut self,
        server_cmd: &ADBCommand,
        mut reader: Box<dyn Read + Send>,
        mut writer: Box<dyn Write + Send>,
    ) -> Result<()> {
        // For bidirectional session, we still need shell features
        // But we can try without checking features first
        self.set_serial_transport()?;
        self.transport.send_adb_request(server_cmd)?;

        let mut read_stream = self.transport.get_raw_connection()?.try_clone()?;
        let mut write_stream = read_stream.try_clone()?;

        // Forward `reader` (that could be stdin e.g) to server socket.
        // This thread may stay blocked on `reader` forever, so it is never joined.
        let (local_result_tx, local_result_rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = match std::io::copy(&mut reader, &mut write_stream) {
                Ok(_) => Ok(()),
                // Stream has been closed by remote, nothing more to do
                Err(e) if e.kind() == ErrorKind::BrokenPipe => Ok(()),
                Err(e) => Err(RustADBError::IOError(e)),
            };

            // Input is exhausted: half-close the socket so that the server ends the session.
            let _ = write_stream.shutdown(std::net::Shutdown::Write);
            if let Err(err) = result {
                let _ = local_result_tx.send(err);
            }
        });

        let mut buffer = vec![0; BUFFER_SIZE].into_boxed_slice();
        let remote_result = loop {
            match read_stream.read(&mut buffer) {
                Ok(0) => break Ok(()),
                Ok(size) => {
                    if let Err(e) = writer
                        .write_all(&buffer[..size])
                        .and_then(|()| writer.flush())
                    {
                        break Err(RustADBError::IOError(e));
                    }
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => break Err(RustADBError::IOError(e)),
            }
        };

        // Make any later write from the input thread fail instead of reaching a closed session
        let _ = read_stream.shutdown(std::net::Shutdown::Both);

        remote_result?;

        // Input thread may still be blocked on `reader`, only report an already known error
        match local_result_rx.try_recv() {
            Ok(e) => Err(e),
            _ => Ok(()),
        }
    }
}
