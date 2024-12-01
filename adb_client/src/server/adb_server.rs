use crate::ADBTransport;
use crate::Result;
use crate::RustADBError;
use crate::TCPServerTransport;
use std::collections::HashMap;
use std::net::SocketAddrV4;
use std::process::Command;

/// Represents an ADB Server
#[derive(Debug, Default)]
pub struct ADBServer {
    /// Internal [TcpStream], lazily initialized
    pub(crate) transport: Option<TCPServerTransport>,
    /// Address to connect to
    pub(crate) socket_addr: Option<SocketAddrV4>,
    /// adb-server start envs
    pub(crate) envs: HashMap<String, String>,
}

impl ADBServer {
    /// Instantiates a new [ADBServer]
    pub fn new(address: SocketAddrV4) -> Self {
        Self {
            transport: None,
            socket_addr: Some(address),
            envs: HashMap::new(),
        }
    }

    /// Returns the current selected transport
    pub(crate) fn get_transport(&mut self) -> Result<&mut TCPServerTransport> {
        self.transport
            .as_mut()
            .ok_or(RustADBError::IOError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "server connection not initialized",
            )))
    }

    /// Connect to underlying transport
    pub(crate) fn connect(&mut self) -> Result<&mut TCPServerTransport> {
        let mut is_local_ip = false;
        let mut transport = if let Some(addr) = &self.socket_addr {
            let ip = addr.ip();
            if ip.is_loopback() || ip.is_unspecified() {
                is_local_ip = true;
            }
            TCPServerTransport::new(*addr)
        } else {
            is_local_ip = true;
            TCPServerTransport::default()
        };

        if is_local_ip {
            // ADB Server is local, we start it if not already running
            let mut command = Command::new("adb");
            command.arg("start-server");
            for (env_k, env_v) in self.envs.iter() {
                command.env(env_k, env_v);
            }

            let child = command.spawn();
            match child {
                Ok(mut child) => {
                    if let Err(e) = child.wait() {
                        log::error!("error while starting adb server: {e}")
                    }
                }
                Err(e) => log::error!("error while starting adb server: {e}"),
            }
        }

        transport.connect()?;
        self.transport = Some(transport);

        self.get_transport()
    }
}

impl Drop for ADBServer {
    fn drop(&mut self) {
        if let Some(ref mut transport) = &mut self.transport {
            let _ = transport.disconnect();
        }
    }
}
