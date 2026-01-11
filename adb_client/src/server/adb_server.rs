use crate::Connected;
use crate::NotConnected;
use crate::Result;
use crate::RustADBError;
use crate::adb_transport::ADBConnectableTransport;
use crate::adb_transport::ADBDisconnectableTransport;
use crate::server::tcp_server_transport::TCPServerTransport;
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::marker::PhantomData;
use std::net::SocketAddrV4;
use std::process::Command;

/// Start an instance of `adb-server`
pub fn start_adb_server<S: BuildHasher>(
    envs: &HashMap<String, String, S>,
    adb_path: &Option<String>,
) {
    // ADB Server is local, we start it if not already running
    let mut command = Command::new(adb_path.as_deref().unwrap_or("adb"));
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

    let child = command.spawn();
    match child {
        Ok(mut child) => {
            if let Err(e) = child.wait() {
                log::error!("error while starting adb server: {e}");
            }
        }
        Err(e) => log::error!("error while starting adb server: {e}"),
    }
}

/// Represents an ADB Server
#[derive(Debug, Default)]
pub struct ADBServer<T> {
    pub(crate) inner_state: PhantomData<T>,
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

impl ADBServer<NotConnected> {
    /// Instantiates a new [`ADBServer`]
    #[must_use]
    pub fn new(address: SocketAddrV4) -> ADBServer<NotConnected> {
        Self {
            inner_state: PhantomData::<NotConnected>,
            transport: None,
            socket_addr: Some(address),
            envs: HashMap::new(),
            adb_path: None,
        }
    }

    /// Instantiates a new [`ADBServer`] with a custom adb path
    #[must_use]
    pub fn new_from_path(
        address: SocketAddrV4,
        adb_path: Option<String>,
    ) -> ADBServer<NotConnected> {
        Self {
            inner_state: PhantomData::<NotConnected>,
            transport: None,
            socket_addr: Some(address),
            envs: HashMap::new(),
            adb_path,
        }
    }

    /// Connect to underlying transport
    pub fn connect(self) -> Result<ADBServer<Connected>> {
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
            start_adb_server(&self.envs, &self.adb_path);
        }

        transport.connect()?;

        let socketaddr = transport.get_socketaddr();

        Ok(ADBServer::<Connected> {
            inner_state: PhantomData,
            transport: Some(transport),
            socket_addr: Some(socketaddr),
            envs: HashMap::default(),
            adb_path: None,
        })
    }
}

impl ADBServer<Connected> {
    /// Returns the current selected transport
    pub(crate) fn get_transport(&mut self) -> Result<&mut TCPServerTransport> {
        self.transport
            .as_mut()
            .ok_or(RustADBError::IOError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "server connection not initialized",
            )))
    }
}

impl<T> Drop for ADBServer<T> {
    fn drop(&mut self) {
        if let Some(transport) = &mut self.transport {
            let _ = transport.disconnect();
        }
    }
}
