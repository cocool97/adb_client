use std::io::BufRead;

use crate::{
    Connected, Result,
    models::{ADBCommand, ADBHostCommand},
    server::{ADBServer, MDNSServices, adb_server::start_adb_server, models::MDNSBackend},
};

const OPENSCREEN_MDNS_BACKEND: &str = "ADB_MDNS_OPENSCREEN";

impl ADBServer<Connected> {
    /// Check if mdns discovery is available
    pub fn mdns_check(&mut self) -> Result<bool> {
        let response = self
            .get_transport()?
            .proxy_connection(&ADBCommand::Host(ADBHostCommand::MDNSCheck), true)?;

        match String::from_utf8(response) {
            Ok(s) if s.starts_with("mdns daemon version") => Ok(true),
            Ok(_) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    /// List all discovered mdns services
    pub fn mdns_services(&mut self) -> Result<Vec<MDNSServices>> {
        let services = self
            .get_transport()?
            .proxy_connection(&ADBCommand::Host(ADBHostCommand::MDNSServices), true)?;

        let mut vec_services: Vec<MDNSServices> = vec![];
        for service in services.lines() {
            match service {
                Ok(service) => {
                    vec_services.push(MDNSServices::try_from(service.as_bytes())?);
                }
                Err(e) => log::error!("{e}"),
            }
        }

        Ok(vec_services)
    }

    /// Check if specified backend mdns service is used, otherwise restart adb server with envs and return the new server instance
    pub fn mdns_force_backend(mut self, backend: MDNSBackend) -> Result<Self> {
        let server_status = self.server_status()?;

        let new_self = if server_status.mdns_backend == backend {
            self
        } else {
            let mut envs = self.envs.clone();
            let server_addr = self.get_transport()?.get_socketaddr();
            let adb_path = self.adb_path.clone();

            envs.insert(
                OPENSCREEN_MDNS_BACKEND.to_string(),
                (if backend == MDNSBackend::OpenScreen {
                    "1"
                } else {
                    "0"
                })
                .to_string(),
            );

            self.kill()?;

            start_adb_server(&envs, &adb_path);

            ADBServer::new_from_path(server_addr, adb_path).connect()?
        };

        Ok(new_self)
    }
}
