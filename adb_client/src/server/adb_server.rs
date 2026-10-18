use crate::ADBTransport;
use crate::Result;
use crate::RustADBError;
use crate::server::tcp_server_transport::TCPServerTransport;
use std::collections::HashMap;
use std::net::{SocketAddrV4, TcpStream};
use std::process::Command;

/// Represents an ADB Server
#[derive(Debug, Default)]
pub struct ADBServer {
    /// Internal [`TcpStream`], lazily initialized
    pub(crate) transport: Option<TCPServerTransport>,
    /// Address to connect to
    pub(crate) socket_addr: Option<SocketAddrV4>,
    /// adb-server start envs
    pub(crate) envs: HashMap<String, String>,
    /// Path to adb binary
    /// If not set, will use adb from PATH
    pub(crate) adb_path: Option<String>,
}

impl ADBServer {
    /// Instantiates a new [`ADBServer`]
    #[must_use]
    pub fn new(address: SocketAddrV4) -> Self {
        Self {
            transport: None,
            socket_addr: Some(address),
            envs: HashMap::new(),
            adb_path: None,
        }
    }

    /// Instantiates a new [`ADBServer`] with a custom adb path
    #[must_use]
    pub fn new_from_path(address: SocketAddrV4, adb_path: Option<String>) -> Self {
        Self {
            transport: None,
            socket_addr: Some(address),
            envs: HashMap::new(),
            adb_path,
        }
    }

    /// Get the server address
    #[must_use]
    pub const fn socket_addr(&self) -> Option<SocketAddrV4> {
        self.socket_addr
    }

    /// Start an instance of `adb-server` by running `adb start-server`.
    ///
    /// `adb_path` is the `adb` binary to use, defaulting to the one found in `PATH`.
    /// Returns an error if the binary cannot be executed, or if it exits unsuccessfully.
    pub fn start(envs: &HashMap<String, String>, adb_path: Option<&str>) -> Result<()> {
        // ADB Server is local, we start it if not already running
        let adb_path = adb_path.unwrap_or("adb");
        let mut command = Command::new(adb_path);
        command.arg("start-server");
        for (env_k, env_v) in envs {
            command.env(env_k, env_v);
        }

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            // Do not show a prompt on Windows
            command.creation_flags(0x08000000);
        }

        let adb_server_status =
            command
                .status()
                .map_err(|source| RustADBError::CannotStartServer {
                    path: adb_path.to_string(),
                    source,
                })?;

        if !adb_server_status.success() {
            return Err(RustADBError::CannotStartServer {
                path: adb_path.to_string(),
                source: std::io::Error::other(format!("`start-server` {adb_server_status}")),
            });
        }

        Ok(())
    }

    /// Returns `true` if a server must be started before using this one: its address is local, and no
    /// server is currently listening on it.
    #[must_use]
    pub fn should_start_server(&self) -> bool {
        self.is_local_server() && TcpStream::connect(self.server_address()).is_err()
    }

    /// Server address, or default one if not set
    fn server_address(&self) -> SocketAddrV4 {
        TCPServerTransport::new_or_default(self.socket_addr).get_socketaddr()
    }

    /// Return whether the server address is a local one
    fn is_local_server(&self) -> bool {
        let ip = *self.server_address().ip();
        ip.is_loopback() || ip.is_unspecified()
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
        let mut transport = TCPServerTransport::new(self.server_address());

        // Only start server if none is running, and if it is a local one
        if let Err(first_err) = transport.connect() {
            if !self.is_local_server() {
                return Err(first_err);
            }
            Self::start(&self.envs, self.adb_path.as_deref())?;
            transport.connect()?;
        }

        self.transport = Some(transport);

        self.get_transport()
    }
}

impl Drop for ADBServer {
    fn drop(&mut self) {
        if let Some(transport) = &mut self.transport {
            let _ = transport.disconnect();
        }
    }
}
