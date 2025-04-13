use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::{sync::mpsc::Sender, thread::JoinHandle};

use crate::{MDNSDevice, Result, RustADBError};

const ADB_SERVICE_NAME: &str = "_adb-tls-connect._tcp.local.";

/// Structure holding responsibility over mdns discovery
pub struct MDNSDiscoveryService {
    daemon: ServiceDaemon,
    thread_handle: Option<JoinHandle<Result<()>>>,
}

impl std::fmt::Debug for MDNSDiscoveryService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MDNSDiscoveryService")
            .field("daemon", &self.daemon.get_metrics())
            .field("handle", &self.thread_handle)
            .finish()
    }
}

impl MDNSDiscoveryService {
    /// Instantiate a new discovery service to find devices over mdns
    pub fn new() -> Result<Self> {
        Ok(MDNSDiscoveryService {
            daemon: ServiceDaemon::new()?,
            thread_handle: None,
        })
    }

    /// Start discovery by spawning a new thread responsible of getting events.
    pub fn start(&mut self, sender: Sender<MDNSDevice>) -> Result<()> {
        let receiver = self.daemon.browse(ADB_SERVICE_NAME)?;

        let handle: JoinHandle<Result<()>> = std::thread::spawn(move || {
            loop {
                while let Ok(event) = receiver.recv() {
                    match event {
                        ServiceEvent::SearchStarted(_)
                        | ServiceEvent::ServiceRemoved(_, _)
                        | ServiceEvent::ServiceFound(_, _)
                        | ServiceEvent::SearchStopped(_) => {
                            // Ignoring these events. We are only interesting in found devices
                            continue;
                        }
                        ServiceEvent::ServiceResolved(service_info) => {
                            if let Err(e) = sender.send(MDNSDevice::from(service_info)) {
                                return Err(e.into());
                            }
                        }
                    }
                }
            }
        });

        self.thread_handle = Some(handle);

        Ok(())
    }

    /// Shutdown discovery engines.
    pub fn shutdown(&mut self) -> Result<()> {
        match self.daemon.shutdown() {
            Ok(_) => Ok(()),
            Err(e) => match e {
                mdns_sd::Error::Again => {
                    self.daemon.shutdown()?;
                    Ok(())
                }
                e => Err(RustADBError::MDNSError(e)),
            },
        }
    }
}
