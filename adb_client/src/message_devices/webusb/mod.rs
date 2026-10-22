//! Async ADB over browser `WebUSB`.
//!
//! Native clients continue to use `ADBDeviceExt`. Browser USB methods
//! return JavaScript promises and cannot implement that blocking trait.
//! See `examples/webusb` for permission, key persistence and shell streaming.

use crate::{Result, message_devices::models::ADBRsaKey};

#[cfg(target_arch = "wasm32")]
mod browser;
#[cfg(any(target_arch = "wasm32", test))]
mod protocol;
#[cfg(target_arch = "wasm32")]
pub use browser::{ADBWebUSBDevice, ADBWebUSBStream, request_device};

/// An in-memory 2048-bit ADB host key. Applications own persistence.
///
/// Its debug representation never includes private key material.
#[derive(Clone)]
pub struct WebUSBKey(ADBRsaKey);

impl std::fmt::Debug for WebUSBKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WebUSBKey { .. }")
    }
}

impl WebUSBKey {
    /// Generate a host key using the operating system or browser CSPRNG.
    /// RSA generation is synchronous; importing a persisted key avoids doing
    /// this work on every connection.
    pub fn generate() -> Result<Self> {
        Ok(Self(ADBRsaKey::new_random()?))
    }

    /// Import a PKCS#8 PEM host key. Only Android's 2048-bit key format is accepted.
    pub fn from_pkcs8_pem(pem: &str) -> Result<Self> {
        let key = ADBRsaKey::new_from_pkcs8(pem)?;
        key.android_pubkey_encode()?;
        Ok(Self(key))
    }

    /// Export the private key for application-managed persistence.
    /// Treat the returned PEM as a credential; do not send it to a server.
    pub fn to_pkcs8_pem(&self) -> Result<String> {
        self.0.to_pkcs8_pem()
    }
}
