use std::collections::HashMap;

use crate::{
    models::{AdbServerCommand, MDNSBackend, MDNSServices},
    ADBServer, Result
};

impl ADBServer {
    /// Check if mdns discovery is available
    pub fn mdns_check(&mut self) -> Result<bool> {
        let response = self
            .connect()?
            .proxy_connection(AdbServerCommand::MDNSCheck, true)?;

        match String::from_utf8(response) {
            Ok(s) if s.starts_with("mdns daemon version") => Ok(true),
            Ok(_) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }
    
    /// List all discovered services
    pub fn mdns_services(&mut self) -> Result<Vec<MDNSServices>> {
        let services = self
        .connect()?
        .proxy_connection(AdbServerCommand::MDNSServices, true)?;

        let mut vec_services: Vec<MDNSServices> = vec![];
        for service in services.split(|x| x.eq(&b'\n')) {
            if service.is_empty() {
                break;
            }

            vec_services.push(MDNSServices::try_from(service.to_vec())?);
        }

        Ok(vec_services)
    }
    
    /// Check if openscreen mdns service is used, otherwise restart adb server with envs
    pub fn mdns_force_openscreen_backend(&mut self) -> Result<()> {
        let status = self.server_status()?;
        if status.mdns_backend != MDNSBackend::OPENSCREEN {
            self.kill()?;
            self.connect_with_envs(Some(HashMap::from([("ADB_MDNS_OPENSCREEN".to_string(), "1".to_string())])))?;
        }

        Ok(())
    }
}
