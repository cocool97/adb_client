use crate::{
    device::adb_message_device::ADBMessageDevice, ADBMessageTransport, AdbStatResponse, Result,
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn stat(&mut self, remote_path: &str) -> Result<AdbStatResponse> {
        let (local_id, remote_id) = self.begin_synchronization()?;
        let adb_stat_response = self.stat_with_explicit_ids(remote_path, local_id, remote_id)?;
        self.end_transaction(local_id, remote_id)?;
        Ok(adb_stat_response)
    }
}
