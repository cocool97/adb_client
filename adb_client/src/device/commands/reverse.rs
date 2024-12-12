use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    sync::mpsc::{self, SyncSender},
    thread::JoinHandle,
};

use crate::{
    constants::BUFFER_SIZE,
    device::{adb_message_device::ADBMessageDevice, ADBTransportMessage, MessageCommand},
    ADBMessageTransport, ADBProtoPort, Result, RustADBError,
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    /// Reverse socket connection
    pub(crate) fn reverse(&mut self, remote: ADBProtoPort, local: ADBProtoPort) -> Result<()> {
        self.open_session(format!("reverse:forward:{remote};{local}\0").as_bytes())?;

        let message = self.get_transport_mut().read_message()?;
        let received_command = message.header().command();
        if received_command != MessageCommand::Write {
            return Err(RustADBError::ADBRequestFailed(format!(
                "expected command WRTE after message, got {}",
                received_command
            )));
        }

        let message = self.get_transport_mut().read_message()?;
        let received_command = message.header().command();
        if received_command != MessageCommand::Clse {
            return Err(RustADBError::ADBRequestFailed(format!(
                "expected command CLSE after message, got {}",
                received_command
            )));
        }

        let mut s = Box::new(self.clone());

        log::debug!("reverse connection setup... waiting for incoming messages");

        let mut active_connections: HashMap<
            u32,
            (SyncSender<ADBTransportMessage>, JoinHandle<Result<()>>),
        > = HashMap::new();
        let mut transport = s.get_transport_mut().clone();

        // Loop dispatching all received messages to corresponding "handlers" threads
        // These threads are stored, and messages are dispatched via channels according to the `local_id` of incoming messages.
        // Finished threads are collected before receiving every new message.
        loop {
            let finished_keys: Vec<u32> = active_connections
                .iter()
                .filter_map(|(&key, (_, handle))| {
                    if handle.is_finished() {
                        Some(key)
                    } else {
                        None
                    }
                })
                .collect();

            for key in finished_keys {
                if let Some((_, handle)) = active_connections.remove(&key) {
                    log::trace!("removing finished thread {key}");
                    if let Err(e) = handle
                        .join()
                        .map_err(|_| RustADBError::ADBRequestFailed("cannot join thread".into()))?
                    {
                        log::error!("error with thread {key}: {e}");
                    }
                }
            }

            let message = transport.read_message()?;
            let remote_id = message.header().arg0();
            log::trace!("received message for {remote_id}");

            match active_connections.get(&remote_id) {
                Some((sender, _)) => {
                    sender.send(message).map_err(|_| RustADBError::SendError)?;
                }
                None => {
                    let (tx, rx) = mpsc::sync_channel(5);
                    let s = s.clone();
                    let handle = std::thread::spawn(move || {
                        let mut ss = s.clone();
                        let received_command = message.header().command();
                        if received_command != MessageCommand::Open {
                            return Err(RustADBError::ADBRequestFailed(format!(
                                "expected command OPEN after message, got {}",
                                received_command
                            )));
                        }

                        ss.set_random_local_id();
                        ss.set_remote_id(message.header().arg0());

                        let local_id = ss.get_local_id()?;
                        let remote_id = ss.get_remote_id()?;
                        ss.get_transport_mut()
                            .write_message(ADBTransportMessage::new(
                                MessageCommand::Okay,
                                local_id,
                                remote_id,
                                &[],
                            ))?;

                        let reverse_dst_proto = ADBProtoPort::try_from(&*message.into_payload())?;

                        log::debug!("Received reverse connection request to {reverse_dst_proto}");

                        let message: ADBTransportMessage = rx.recv()?;

                        let request_data = message.into_payload();

                        ss.get_transport_mut()
                            .write_message(ADBTransportMessage::new(
                                MessageCommand::Okay,
                                local_id,
                                remote_id,
                                &[],
                            ))?;

                        match reverse_dst_proto {
                            ADBProtoPort::TCP(port) => {
                                let addr: SocketAddr = ([127, 0, 0, 1], port).into();
                                let mut tcp_stream = TcpStream::connect(addr)?;
                                tcp_stream.write_all(&request_data)?;

                                loop {
                                    let mut buffer = [0; BUFFER_SIZE];
                                    let amount_read = tcp_stream.read(&mut buffer);

                                    match amount_read {
                                        Ok(0) => break,
                                        Ok(v) => {
                                            ss.get_transport_mut().write_message(
                                                ADBTransportMessage::new(
                                                    MessageCommand::Write,
                                                    local_id,
                                                    remote_id,
                                                    &buffer[..v],
                                                ),
                                            )?;
                                            rx.recv()?;
                                        }
                                        Err(e) => return Err(RustADBError::IOError(e)),
                                    }
                                }
                            }
                        };

                        ss.get_transport_mut()
                            .write_message(ADBTransportMessage::new(
                                MessageCommand::Clse,
                                local_id,
                                remote_id,
                                &[],
                            ))?;

                        rx.recv()?;

                        Ok(())
                    });

                    active_connections.insert(remote_id, (tx, handle));
                }
            }
        }
    }

    /// Remove all previously applied reverse rules
    pub(crate) fn reverse_remove_all(&mut self) -> Result<()> {
        self.open_session(b"reverse:killforward-all\0")?;
        let message = self.get_transport_mut().read_message()?;
        let received_command = message.header().command();
        if received_command != MessageCommand::Write {
            return Err(RustADBError::ADBRequestFailed(format!(
                "expected command WRTE after message, got {}",
                received_command
            )));
        }

        let message = self.get_transport_mut().read_message()?;
        let received_command = message.header().command();
        if received_command != MessageCommand::Clse {
            return Err(RustADBError::ADBRequestFailed(format!(
                "expected command CLSE after message, got {}",
                received_command
            )));
        }

        Ok(())
    }
}
