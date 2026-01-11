use crate::{
    Connected, Result,
    models::{ADBCommand, ADBHostCommand},
    server::{ADBServer, WaitForDeviceState, WaitForDeviceTransport},
};

impl ADBServer<Connected> {
    /// Wait for a device in a given state to be connected
    pub fn wait_for_device(
        &mut self,
        state: WaitForDeviceState,
        transport: Option<WaitForDeviceTransport>,
    ) -> Result<()> {
        let transport = transport.unwrap_or_default();

        self.get_transport()?.send_adb_request(&ADBCommand::Host(
            ADBHostCommand::WaitForDevice(state, transport),
        ))?;

        // Server should respond with an "OKAY" response
        self.get_transport()?.read_adb_response()
    }
}
