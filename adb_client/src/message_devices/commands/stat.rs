use crate::{
    AdbStatResponse, Result,
    message_devices::{
        adb_message_device::ADBMessageDevice, adb_message_transport::ADBMessageTransport,
    },
};
use std::path::Path;

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn stat<P: AsRef<Path>>(&mut self, remote_path: P) -> Result<AdbStatResponse> {
        let mut session = self.open_synchronization_session()?;
        let adb_stat_response =
            session.stat_with_explicit_ids(&remote_path.as_ref().display().to_string())?;
        self.end_transaction(&mut session)?;
        Ok(adb_stat_response)
    }
}
