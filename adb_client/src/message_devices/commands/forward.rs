use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};

use crate::{
    Result, RustADBError,
    message_devices::{
        adb_message_device::ADBMessageDevice,
        adb_message_transport::ADBMessageTransport,
        adb_session::ADBSessionOpener,
        commands::utils::{bridge, loopback_addresses, parse_tcp_endpoint},
    },
    models::ADBLocalCommand,
};

/// How long to wait before accepting again after a failure, e.g. when out of file descriptors.
const ACCEPT_RETRY_DELAY: Duration = Duration::from_millis(100);

/// Maximum time to connect to a listener, to wake its thread up when stopping.
const WAKE_UP_TIMEOUT: Duration = Duration::from_secs(1);

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    /// Forward connections to host `local` endpoint to device `remote` endpoint, until removed or the device is
    /// dropped. Replaces the previous forward of `local`. Return the host endpoint listened on: `tcp:0` is resolved
    /// to the port allocated.
    pub(crate) fn forward(&self, remote: &str, local: &str) -> Result<String> {
        let port = parse_tcp_endpoint(local)?;

        let mut forwards = self.forwards()?;
        // Release the port before binding it again
        if port != 0 {
            forwards.remove(&port);
        }
        let listener = ForwardListener::start(port, remote, &self.session_opener())?;
        let port = listener.port;
        forwards.insert(port, listener);
        drop(forwards);
        Ok(format!("tcp:{port}"))
    }

    /// Remove the forward of host `local` endpoint. Connections already forwarded are kept.
    pub(crate) fn forward_remove(&self, local: &str) -> Result<()> {
        let port = parse_tcp_endpoint(local)?;
        self.forwards()?
            .remove(&port)
            .map(drop)
            .ok_or_else(|| RustADBError::ADBRequestFailed(format!("listener '{local}' not found")))
    }

    /// Remove every forward. Connections already forwarded are kept.
    pub(crate) fn forward_remove_all(&self) -> Result<()> {
        self.forwards()?.clear();
        Ok(())
    }
}

/// Listen on a host port, and relay each connection to a device endpoint. Stops listening when dropped.
#[derive(Debug)]
pub struct ForwardListener {
    port: u16,
    stop: Arc<AtomicBool>,
    /// Accepting thread of each address listened on.
    threads: Vec<(SocketAddr, JoinHandle<()>)>,
}

impl ForwardListener {
    fn start<T: ADBMessageTransport>(
        port: u16,
        remote: &str,
        opener: &ADBSessionOpener<T>,
    ) -> Result<Self> {
        let listeners = Self::bind(port)?;

        let mut forward = Self {
            port: listeners[0].local_addr()?.port(),
            stop: Arc::new(AtomicBool::new(false)),
            threads: Vec::new(),
        };
        // On failure, dropping `forward` stops the threads already started
        for listener in listeners {
            let address = listener.local_addr()?;
            let (remote, opener, stop) = (remote.to_string(), opener.clone(), forward.stop.clone());
            let thread = std::thread::Builder::new()
                .name(format!("adb-forward-{address}"))
                .spawn(move || Self::accept_connections(&listener, &remote, &opener, &stop))?;
            forward.threads.push((address, thread));
        }
        Ok(forward)
    }

    /// Like adb, listen on IPv4 and IPv6 loopback: only local clients can connect, through either. Listening on one
    /// is enough.
    fn bind(port: u16) -> Result<Vec<TcpListener>> {
        let [ipv4, ipv6] = loopback_addresses(port);
        match TcpListener::bind(ipv4) {
            Ok(ipv4_listener) => {
                // Same port on both, even if allocated
                let mut ipv6 = ipv6;
                ipv6.set_port(ipv4_listener.local_addr()?.port());
                let mut listeners = vec![ipv4_listener];
                match TcpListener::bind(ipv6) {
                    Ok(ipv6_listener) => listeners.push(ipv6_listener),
                    Err(e) => log::debug!("cannot listen on {ipv6}: {e}"),
                }
                Ok(listeners)
            }
            Err(e) => TcpListener::bind(ipv6)
                .map(|ipv6_listener| vec![ipv6_listener])
                .map_err(|_| e.into()),
        }
    }

    fn accept_connections<T: ADBMessageTransport>(
        listener: &TcpListener,
        remote: &str,
        opener: &ADBSessionOpener<T>,
        stop: &AtomicBool,
    ) {
        for stream in listener.incoming() {
            // Woken up by a connection when stopping
            if stop.load(Ordering::SeqCst) {
                return;
            }

            let stream = match stream {
                Ok(stream) => stream,
                Err(e) => {
                    // Keep listening, failures are specific to a connection or temporary
                    log::warn!("cannot accept connection forwarded to {remote}: {e}");
                    std::thread::sleep(ACCEPT_RETRY_DELAY);
                    continue;
                }
            };

            // Each connection gets its own session, opened without blocking other connections
            let opener = opener.clone();
            let command = ADBLocalCommand::Connect(remote.to_string());
            let spawned = std::thread::Builder::new()
                .name("adb-forward-connection".into())
                .spawn(move || match opener.open(&command) {
                    Ok(session) => bridge(session, &stream),
                    Err(e) => log::warn!("cannot forward connection to {command}: {e}"),
                });
            if let Err(e) = spawned {
                log::warn!("cannot forward connection to {remote}: {e}");
            }
        }
    }
}

impl Drop for ForwardListener {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        for (address, thread) in self.threads.drain(..) {
            // Wake the thread up, blocked accepting connections
            if let Err(e) = TcpStream::connect_timeout(&address, WAKE_UP_TIMEOUT) {
                log::warn!("{address} keeps listening until its next connection: {e}");
                continue;
            }
            if thread.join().is_err() {
                log::error!("forward listener thread panicked");
            }
        }
    }
}
