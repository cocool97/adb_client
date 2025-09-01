use crate::{
    AdbStatResponse, Result,
    message_devices::{
        adb_message_device::ADBMessageDevice, adb_message_transport::ADBMessageTransport,
    },
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn stat(&mut self, remote_path: &str) -> Result<AdbStatResponse> {
        self.begin_synchronization()?;
        let adb_stat_response = self.stat_with_explicit_ids(remote_path)?;
        self.end_transaction()?;
        Ok(adb_stat_response)
    }
}
