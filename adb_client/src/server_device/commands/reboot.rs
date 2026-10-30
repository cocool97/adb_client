use crate::{
    Result,
    models::{ADBCommand, ADBLocalCommand, RebootType},
    server_device::ADBServerDevice,
};

impl ADBServerDevice {
    /// Reboots the device
    pub fn reboot(&self, reboot_type: RebootType) -> Result<()> {
        // adbd restarts, and may then advertise different features
        self.invalidate_features();
        let transport = self.connect_to_device()?;

        transport
            .proxy_connection(
                &ADBCommand::Local(ADBLocalCommand::Reboot(reboot_type)),
                false,
            )
            .map(|_| ())
    }
}
