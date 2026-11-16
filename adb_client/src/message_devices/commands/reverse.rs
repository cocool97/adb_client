use std::net::TcpStream;

use crate::{
    Result, RustADBError,
    message_devices::{
        adb_message_device::ADBMessageDevice,
        adb_message_transport::ADBMessageTransport,
        adb_session::ADBSession,
        commands::utils::{bridge, loopback_addresses, parse_tcp_endpoint},
    },
    models::ADBLocalCommand,
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    /// Forward connections to device `remote` endpoint to host `local` endpoint, until removed or the device is
    /// disconnected. Replaces the previous reverse forward of `remote`.
    pub(crate) fn reverse(&self, remote: &str, local: &str) -> Result<()> {
        parse_tcp_endpoint(local)?;

        // Accepted before asking, the device may connect as soon as it answers
        let dispatcher = self.dispatcher().handle();
        let previous = dispatcher
            .reverses()?
            .insert(remote.to_string(), local.to_string());

        self.reverse_request(&ADBLocalCommand::Reverse(
            remote.to_string(),
            local.to_string(),
        ))
        .inspect_err(|_| {
            // The device keeps its previous reverse forward of `remote`, if any
            if let Ok(mut reverses) = dispatcher.reverses() {
                match previous {
                    Some(previous) => reverses.insert(remote.to_string(), previous),
                    None => reverses.remove(remote),
                };
            }
        })
    }

    /// Remove the reverse forward of device `remote` endpoint.
    pub(crate) fn reverse_remove(&self, remote: &str) -> Result<()> {
        self.reverse_request(&ADBLocalCommand::ReverseRemove(remote.to_string()))?;
        self.dispatcher().handle().reverses()?.remove(remote);
        Ok(())
    }

    /// Remove every reverse forward.
    pub(crate) fn reverse_remove_all(&self) -> Result<()> {
        self.reverse_request(&ADBLocalCommand::ReverseRemoveAll)?;
        self.dispatcher().handle().reverses()?.clear();
        Ok(())
    }

    /// Send a `reverse:` request, and check the device answer.
    fn reverse_request(&self, command: &ADBLocalCommand) -> Result<()> {
        let mut session = self.open_session(command)?;
        let response = session.recv_until_close()?;

        // `OKAY`, or `FAIL` followed by the hexadecimal length of the reason and the reason
        match response.get(..4) {
            Some(b"OKAY") => Ok(()),
            Some(b"FAIL") => Err(RustADBError::ADBRequestFailed(
                String::from_utf8_lossy(response.get(8..).unwrap_or_default()).into_owned(),
            )),
            _ => Err(RustADBError::ADBRequestFailed(format!(
                "unexpected answer to {command}: {}",
                String::from_utf8_lossy(&response)
            ))),
        }
    }
}

/// Connect a stream opened by the device for a reverse forward to its host `local` endpoint, and relay it.
///
/// `session` is accepted once connected, or refused if connecting fails.
pub fn accept_reverse_connection<T: ADBMessageTransport>(
    mut session: ADBSession<T>,
    local: String,
) {
    let spawned = std::thread::Builder::new()
        .name("adb-reverse-connection".into())
        .spawn(move || {
            // Like adb, try IPv4 then IPv6 loopback
            let stream = parse_tcp_endpoint(&local)
                .and_then(|port| Ok(TcpStream::connect(&loopback_addresses(port)[..])?));
            // Dropping the session refuses the stream
            match stream {
                Ok(stream) => match session.accept() {
                    Ok(()) => bridge(session, &stream),
                    Err(e) => log::warn!("cannot accept reverse connection to {local}: {e}"),
                },
                Err(e) => log::warn!("cannot connect reverse connection to {local}: {e}"),
            }
        });
    if let Err(e) = spawned {
        log::warn!("cannot accept reverse connection: {e}");
    }
}
