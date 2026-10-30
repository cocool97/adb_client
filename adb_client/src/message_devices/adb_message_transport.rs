use std::time::Duration;

use crate::{
    Result, adb_transport::ADBTransport,
    message_devices::adb_transport_message::ADBTransportMessage,
};

const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(u64::MAX);
const DEFAULT_WRITE_TIMEOUT: Duration = Duration::from_secs(2);

/// Maximum time to wait for the rest of a message, once its first bytes have been received.
pub const MESSAGE_END_TIMEOUT: Duration = Duration::from_secs(10);

/// Trait representing a transport able to read and write messages.
pub trait ADBMessageTransport: ADBTransport + Clone + Send + 'static {
    /// An upgrade of the connection has been asked by the device.
    /// Some transports may not need this feature, a blanket implementation is provided as default implementation.
    fn upgrade_connection(&mut self) -> Result<()> {
        log::trace!("not upgrade needed for this transport");
        Ok(())
    }

    /// Read a message, waiting at most `read_timeout` for it to start.
    ///
    /// Once its first bytes are received, the message is read entirely (waiting at most [`MESSAGE_END_TIMEOUT`]
    /// between two chunks). A timeout before the message starts leaves the transport usable.
    ///
    /// Only one message can be read at a time, but messages can be written while a message is being read.
    fn read_message_with_timeout(&mut self, read_timeout: Duration) -> Result<ADBTransportMessage>;

    /// Read data to underlying connection, using default timeout
    fn read_message(&mut self) -> Result<ADBTransportMessage> {
        self.read_message_with_timeout(DEFAULT_READ_TIMEOUT)
    }

    /// Write a message using given timeout on the underlying transport
    fn write_message_with_timeout(
        &mut self,
        message: ADBTransportMessage,
        write_timeout: Duration,
    ) -> Result<()>;

    /// Write data to underlying connection, using default timeout
    fn write_message(&mut self, message: ADBTransportMessage) -> Result<()> {
        self.write_message_with_timeout(message, DEFAULT_WRITE_TIMEOUT)
    }
}
