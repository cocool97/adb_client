use std::{fs::File, io::Read, path::Path};

use crate::{
    models::AdbServerCommand, server_device::ADBServerDevice, utils::check_extension_is_apk, Result,
};

impl ADBServerDevice {
    /// Install an APK on device
    pub fn install<P: AsRef<Path>>(&mut self, apk_path: P) -> Result<()> {
        let mut apk_file = File::open(&apk_path)?;

        check_extension_is_apk(&apk_path)?;

        let file_size = apk_file.metadata()?.len();

        let serial: String = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbServerCommand::TransportSerial(serial))?;

        self.get_transport_mut()
            .send_adb_request(AdbServerCommand::Install(file_size))?;

        let mut raw_connection = self.get_transport_mut().get_raw_connection()?;

        std::io::copy(&mut apk_file, &mut raw_connection)?;

        let mut data = [0; 1024];
        let read_amount = self.get_transport().get_raw_connection()?.read(&mut data)?;

        match &data[0..read_amount] {
            b"Success\n" => {
                log::info!(
                    "APK file {} successfully installed",
                    apk_path.as_ref().display()
                );
                Ok(())
            }
            d => Err(crate::RustADBError::ADBRequestFailed(String::from_utf8(
                d.to_vec(),
            )?)),
        }
    }
}
