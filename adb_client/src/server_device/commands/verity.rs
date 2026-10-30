use crate::{
    Result,
    models::{ADBCommand, ADBLocalCommand},
    server_device::ADBServerDevice,
};

impl ADBServerDevice {
    /// Disable verity on the device
    pub fn disable_verity(&self) -> Result<()> {
        let transport = self.connect_to_device()?;

        transport.send_adb_request(&ADBCommand::Local(ADBLocalCommand::DisableVerity))
    }

    /// Enable verity on the device
    pub fn enable_verity(&self) -> Result<()> {
        let transport = self.connect_to_device()?;

        transport.send_adb_request(&ADBCommand::Local(ADBLocalCommand::EnableVerity))
    }
}
