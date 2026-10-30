use crate::{
    Result,
    models::{ADBCommand, ADBLocalCommand},
    server_device::ADBServerDevice,
};

impl ADBServerDevice {
    /// Set adb daemon to tcp/ip mode
    pub fn tcpip(&self, port: u16) -> Result<()> {
        // adbd restarts, and may then advertise different features
        self.invalidate_features();
        let transport = self.connect_to_device()?;

        transport
            .proxy_connection(&ADBCommand::Local(ADBLocalCommand::TcpIp(port)), false)
            .map(|_| ())
    }
}
