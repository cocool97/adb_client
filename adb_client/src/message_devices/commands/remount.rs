use crate::{
    Result,
    message_devices::{
        adb_message_device::ADBMessageDevice, adb_message_transport::ADBMessageTransport,
    },
    models::{ADBLocalCommand, RemountInfo},
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn remount(&self) -> Result<Vec<RemountInfo>> {
        let output = self
            .open_session(&ADBLocalCommand::Remount)?
            .recv_until_close()?;

        RemountInfo::from_str_response(String::from_utf8_lossy(&output).trim())
    }
}
