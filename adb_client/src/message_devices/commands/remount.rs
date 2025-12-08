use crate::{
    Result, RustADBError,
    message_devices::{
        adb_message_device::ADBMessageDevice, adb_message_transport::ADBMessageTransport,
        message_commands::MessageCommand,
    },
    models::RemountInfo,
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn remount(&mut self) -> Result<Vec<RemountInfo>> {
        self.open_session(b"remount:\0")?;

        let response = self.get_transport_mut().read_message()?;

        if response.header().command() != MessageCommand::Okay {
            return Err(RustADBError::ADBRequestFailed(format!(
                "wrong command {}",
                response.header().command()
            )));
        }

        let mut response_str: Vec<String> = Vec::new();

        loop {
            let response = self.get_transport_mut().read_message()?;

            if response.header().command() != MessageCommand::Write {
                break;
            }

            let payload_str = String::from_utf8_lossy(response.payload());
            let payload_str = payload_str.trim();

            response_str.push(payload_str.to_string());
        }

        RemountInfo::from_str_response(&response_str.join("\n"))
    }
}
