use crate::{
    ADBMessageTransport, AdbStatResponse, Result, device::adb_message_device::ADBMessageDevice,
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn stat(&mut self, remote_path: &str) -> Result<AdbStatResponse> {
        let session = self.begin_synchronization()?;
        let adb_stat_response = self.stat_with_explicit_ids(session, remote_path)?;
        self.end_transaction(session)?;
        Ok(adb_stat_response)
    }
}
