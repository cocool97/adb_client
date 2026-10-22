//! Small browser UI adapter for the async `WebUSB` API.
#![cfg(target_arch = "wasm32")]
#![forbid(unsafe_code)]

use adb_client::webusb::{ADBWebUSBDevice, WebUSBKey};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

fn error(e: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&e.to_string())
}

/// Request USB access synchronously from the Connect button's handler.
///
/// # Errors
/// Returns an error if the browser does not provide USB access.
#[wasm_bindgen]
pub fn request() -> std::result::Result<js_sys::Promise<web_sys::UsbDevice>, JsValue> {
    adb_client::webusb::request_device().map_err(error)
}

/// A connected browser client. The UI serializes calls to this object.
#[wasm_bindgen]
pub struct Client(Option<ADBWebUSBDevice>);

#[wasm_bindgen]
impl Client {
    /// Open the selected device and authenticate with an in-memory PKCS#8 key.
    ///
    /// # Errors
    /// Rejects invalid keys, USB failures, and unsuccessful authentication.
    pub async fn connect(device: JsValue, pem: String) -> std::result::Result<Self, JsValue> {
        let key = WebUSBKey::from_pkcs8_pem(&pem).map_err(error)?;
        let device = ADBWebUSBDevice::connect(device.unchecked_into(), &key)
            .await
            .map_err(error)?;
        Ok(Self(Some(device)))
    }

    /// Stream a shell's combined output to a JS callback as `Uint8Array` chunks.
    ///
    /// # Errors
    /// Rejects protocol/USB failures, callback exceptions, or a closed client.
    pub async fn shell(
        &mut self,
        command: String,
        output: js_sys::Function,
    ) -> std::result::Result<(), JsValue> {
        let device = self.0.as_mut().ok_or_else(|| error("Not connected"))?;
        device
            .shell_command(&command, |bytes| {
                output
                    .call1(&JsValue::UNDEFINED, &js_sys::Uint8Array::from(bytes))
                    .map_err(|e| {
                        adb_client::RustADBError::ADBRequestFailed(format!(
                            "Browser output callback: {e:?}"
                        ))
                    })?;
                Ok(())
            })
            .await
            .map_err(error)
    }

    /// Release the USB interface.
    ///
    /// # Errors
    /// Rejects if the browser fails to close the device.
    pub async fn disconnect(&mut self) -> std::result::Result<(), JsValue> {
        if let Some(device) = self.0.take() {
            device.disconnect().await.map_err(error)?;
        }
        Ok(())
    }
}
