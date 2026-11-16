use std::{
    io::{Read, Write},
    net::{Shutdown, TcpStream},
    sync::mpsc::{self, Sender},
};

use crate::{
    Result, RustADBError,
    message_devices::{
        adb_message_transport::ADBMessageTransport, adb_session::ADBSession,
        message_commands::MessageCommand,
    },
};

/// Relay data between `session` and `stream` both ways, until either side closes. Blocks until then.
///
/// Data received from the device is written to `stream` from the current thread, and data read from `stream`
/// is sent to the device from a background thread, one message at a time.
pub fn bridge<T: ADBMessageTransport>(mut session: ADBSession<T>, stream: &TcpStream) {
    let mut stream_reader = match stream.try_clone() {
        Ok(stream_reader) => stream_reader,
        Err(e) => {
            log::warn!("cannot relay connection: {e}");
            return;
        }
    };

    // Device acknowledgements of the data sent
    let (acks, acks_receiver) = mpsc::channel();
    let mut writer = session.writer();
    let max_payload_size = session.max_payload_size();
    let writer_thread = std::thread::Builder::new()
        .name("adb-bridge-writer".into())
        .spawn(move || {
            let mut buffer = vec![0; max_payload_size];
            loop {
                let size = match stream_reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(size) => size,
                    Err(e) => {
                        log::debug!("cannot read relayed connection: {e}");
                        break;
                    }
                };
                // Wait for the device to be ready before sending more
                if let Err(e) = writer.write(&buffer[..size]) {
                    log::debug!("cannot send relayed data to device: {e}");
                    break;
                }
                if acks_receiver.recv().is_err() {
                    break;
                }
            }

            // Best effort, the session also stops receiving
            let _ = writer.close();
        });
    let writer_thread = match writer_thread {
        Ok(writer_thread) => writer_thread,
        Err(e) => {
            log::warn!("cannot relay connection: {e}");
            return;
        }
    };

    if let Err(e) = relay_from_device(&mut session, stream, &acks) {
        log::debug!("relayed session ended: {e}");
    }

    // Stop the writer thread, whether reading or waiting for an acknowledgement, and wait for it: it must not send
    // data once the session is closed
    let _ = stream.shutdown(Shutdown::Both);
    drop(acks);
    if writer_thread.join().is_err() {
        log::error!("relay writer thread panicked");
    }
}

/// Write data received on `session` into `stream`, until the session is closed.
fn relay_from_device<T: ADBMessageTransport>(
    session: &mut ADBSession<T>,
    mut stream: &TcpStream,
    acks: &Sender<()>,
) -> Result<()> {
    loop {
        let message = session.recv()?;
        match message.header().command() {
            MessageCommand::Write => {
                stream.write_all(message.payload())?;
                // Acknowledge once written, so a slow reader slows the device down
                session.send_okay()?;
            }
            MessageCommand::Okay => {
                // The writer thread may already be gone
                let _ = acks.send(());
            }
            MessageCommand::Clse => return Ok(()),
            command => {
                return Err(RustADBError::ADBRequestFailed(format!(
                    "unexpected {command} message received during session"
                )));
            }
        }
    }
}
