use std::{
    collections::{HashMap, VecDeque},
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};

use crate::{
    Result,
    adb_transport::ADBTransport,
    message_devices::{
        adb_message_transport::ADBMessageTransport, adb_transport_message::ADBTransportMessage,
    },
};

/// Internal structure handling multiplexing of messages over various sessions.
///
/// It spawns a thread reading the transport, and implements [`ADBMessageTransport`] to read / write messages..
#[derive(Clone, Debug)]
pub(crate) struct ADBMessageMultiplexer<T: ADBMessageTransport> {
    transport: T,
    authenticated_data: Arc<RwLock<HashMap<u32, VecDeque<ADBTransportMessage>>>>,
    unauthenticated_data: Arc<RwLock<VecDeque<ADBTransportMessage>>>,
    handle: Option<Arc<JoinHandle<Result<()>>>>,
    authenticated: Arc<AtomicBool>,
}

impl<T: ADBMessageTransport> ADBMessageMultiplexer<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            authenticated_data: Arc::default(),
            unauthenticated_data: Arc::default(),
            handle: None,
            authenticated: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn upgrade_connection(&mut self) -> Result<()> {
        self.transport.upgrade_connection()
    }

    pub fn set_authenticated(&mut self) {
        log::debug!("multiplexer: authenticated");
        self.authenticated.store(true, Ordering::Relaxed);
    }

    pub(crate) fn read_authentication_message(&mut self) -> Result<ADBTransportMessage> {
        self.read_message_with_timeout(None, Duration::from_secs(u64::MAX))
    }

    pub(crate) fn read_message(&mut self, local_id: u32) -> Result<ADBTransportMessage> {
        self.read_message_with_timeout(Some(local_id), Duration::from_secs(u64::MAX))
    }

    pub(crate) fn write_message(&mut self, message: ADBTransportMessage) -> Result<()> {
        self.write_message_with_timeout(message, Duration::from_secs(2))
    }

    pub fn read_message_with_timeout(
        &mut self,
        local_id: Option<u32>,
        read_timeout: std::time::Duration,
    ) -> Result<ADBTransportMessage> {
        loop {
            if let Some(local_id) = local_id {
                let mut rw_data = self.authenticated_data.write()?;

                if let Some(d) = rw_data.get_mut(&local_id)
                    && let Some(v) = d.pop_front()
                {
                    return Ok(v);
                }
            } else {
                let mut rw_data = self.unauthenticated_data.write()?;
                if let Some(v) = rw_data.pop_front() {
                    return Ok(v);
                }
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn write_message_with_timeout(
        &mut self,
        message: ADBTransportMessage,
        write_timeout: std::time::Duration,
    ) -> Result<()> {
        self.transport
            .write_message_with_timeout(message, write_timeout)
    }
}

impl<T: ADBMessageTransport> ADBTransport for ADBMessageMultiplexer<T> {
    fn connect(&mut self) -> crate::Result<()> {
        self.transport.connect()?;

        let data = self.authenticated_data.clone();
        let unauth_data = self.unauthenticated_data.clone();
        let mut transport = self.transport.clone();
        let authenticated = self.authenticated.clone();

        // Spawn a thread responsible of continously reading the underlying transport
        // and pushing messages to the internal data structure
        let handle = std::thread::spawn(move || {
            loop {
                log::trace!("waiting for incoming message");
                let message = transport.read_message()?;

                let remote_id = message.header().arg1();

                if authenticated.load(Ordering::Relaxed) {
                    log::trace!("got new authenticated message for {remote_id} session");

                    let mut rw_data = data.write()?;
                    let new_value = if let Some(mut d) = rw_data.remove(&remote_id) {
                        d.push_back(message);
                        d
                    } else {
                        let mut v = VecDeque::new();
                        v.push_back(message);
                        v
                    };
                    rw_data.insert(remote_id, new_value);
                } else {
                    log::trace!("got new pre-authenticated message");
                    let mut rw_unauth_data = unauth_data.write()?;
                    rw_unauth_data.push_back(message);
                }
            }
        });

        self.handle = Some(Arc::new(handle));

        Ok(())
    }

    fn disconnect(&mut self) -> crate::Result<()> {
        // Empty both internal data storage structures
        {
            let mut rw_data = self.authenticated_data.write()?;
            *rw_data = HashMap::default();
        }

        {
            let mut rw_unauth_data = self.unauthenticated_data.write()?;
            *rw_unauth_data = VecDeque::default();
        }

        if let Some(handle) = self.handle.take()
            && let Some(handle) = Arc::into_inner(handle)
            && let Err(e) = handle.join()
        {
            log::error!("Error joining multiplexer thread: {e:?}");
        }

        Ok(())
    }
}
