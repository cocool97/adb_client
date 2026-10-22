use super::{
    WebUSBKey,
    protocol::{self, Connection, Stream, Transport},
};
use crate::{Result, RustADBError};
use js_sys::{Promise, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{
    UsbAlternateInterface, UsbConfiguration, UsbDevice, UsbDeviceFilter, UsbDeviceRequestOptions,
    UsbDirection, UsbEndpointType, UsbInterface, UsbTransferStatus,
};

#[allow(clippy::needless_pass_by_value)] // Matches Promise/map_err error callbacks.
fn js_error(value: JsValue) -> RustADBError {
    RustADBError::ADBRequestFailed(format!("WebUSB: {value:?}"))
}

/// Open the browser device picker immediately, from a click or key handler.
///
/// Await the returned promise using [`wasm_bindgen_futures::JsFuture`]. Only
/// interfaces matching ADB (ff/42/01) are offered. HTTPS or localhost, `WebUSB`
/// support, USB debugging, and the phone's RSA authorization are required.
/// The promise can reject if the picker is cancelled or access is denied.
/// Calling this function does not open or claim a device.
pub fn request_device() -> Result<Promise<UsbDevice>> {
    let window =
        web_sys::window().ok_or_else(|| protocol::invalid("WebUSB requires a browser window"))?;
    let usb = window.navigator().usb();
    if usb.is_undefined() || usb.is_null() {
        return Err(protocol::invalid("This browser does not support WebUSB"));
    }
    let filter = UsbDeviceFilter::new();
    filter.set_class_code(0xff);
    filter.set_subclass_code(0x42);
    filter.set_protocol_code(0x01);
    Ok(usb.request_device(&UsbDeviceRequestOptions::new(&[filter])))
}

/// An authenticated ADB connection to a browser-selected Android device.
///
/// One logical stream is allowed at a time. Dropping an unfinished stream or
/// cancelling an I/O future invalidates that connection; call `disconnect`
/// and connect again. Dropping the device schedules best-effort USB cleanup.
/// This is an async API, independent of the native blocking `ADBDeviceExt`.
#[derive(Debug)]
pub struct ADBWebUSBDevice {
    connection: Connection<WebUSBTransport>,
}

impl ADBWebUSBDevice {
    /// Open the selected device, claim its ADB interface and authenticate.
    /// The key stays in application memory and is not written to a filesystem.
    /// Awaiting authentication can wait for the phone's authorization dialog;
    /// dropping this future closes the USB handle on a best-effort basis.
    pub async fn connect(device: UsbDevice, key: &WebUSBKey) -> Result<Self> {
        let transport = WebUSBTransport::open(device).await?;
        Ok(Self {
            connection: Connection::connect(transport, key).await?,
        })
    }

    /// Open an ADB service such as `shell:...`, `sync:` or `logcat` via a shell.
    /// Names must not contain NUL bytes. The terminating NUL is added here.
    pub async fn open(&mut self, service: &str) -> Result<ADBWebUSBStream<'_>> {
        Ok(ADBWebUSBStream {
            inner: self.connection.open(service).await?,
        })
    }

    /// Run a legacy shell command, streaming combined stdout and stderr.
    /// This does not report an exit status; shell v2 is not advertised.
    /// An error from the output callback invalidates the connection.
    pub async fn shell_command(
        &mut self,
        command: &str,
        mut output: impl FnMut(&[u8]) -> Result<()>,
    ) -> Result<()> {
        let mut stream = self.open(&format!("shell:{command}")).await?;
        while let Some(bytes) = stream.read().await? {
            output(&bytes)?;
        }
        Ok(())
    }

    /// Close the USB device, releasing its claimed interface and pending I/O.
    pub async fn disconnect(mut self) -> Result<()> {
        self.connection.transport.close().await
    }
}

/// A single, exclusively borrowed ADB service stream.
///
/// Finish with `close()` or read until `None` before reusing the device.
/// Dropping it early invalidates the connection. Read and write cancellation
/// also require disconnect/reconnect; `WebUSB` cannot cancel individual reads.
#[derive(Debug)]
pub struct ADBWebUSBStream<'a> {
    inner: Stream<'a, WebUSBTransport>,
}

impl ADBWebUSBStream<'_> {
    /// Receive one bounded ADB payload and acknowledge it. `None` means closed.
    pub async fn read(&mut self) -> Result<Option<Vec<u8>>> {
        self.inner.read().await
    }

    /// Send bytes in negotiated payload-sized chunks, awaiting each OKAY.
    /// One crossed incoming packet may be retained until the next `read()`.
    pub async fn write(&mut self, bytes: &[u8]) -> Result<()> {
        self.inner.write(bytes).await
    }

    /// Close the service and wait for its close acknowledgement.
    pub async fn close(mut self) -> Result<()> {
        self.inner.close().await
    }
}

#[derive(Debug, Clone, Copy)]
struct Endpoints {
    configuration: u8,
    interface: u8,
    alternate: u8,
    input: u8,
    output: u8,
    output_packet: u32,
}

fn endpoints(configuration: &UsbConfiguration) -> Option<Endpoints> {
    for interface in configuration.interfaces().iter() {
        let interface: UsbInterface = interface.unchecked_into();
        for alternate in interface.alternates().iter() {
            let alternate: UsbAlternateInterface = alternate.unchecked_into();
            if alternate.interface_class() != 0xff
                || alternate.interface_subclass() != 0x42
                || alternate.interface_protocol() != 0x01
            {
                continue;
            }
            let mut input = None;
            let mut output = None;
            for endpoint in alternate.endpoints().iter() {
                let endpoint: web_sys::UsbEndpoint = endpoint.unchecked_into();
                if endpoint.type_() != UsbEndpointType::Bulk {
                    continue;
                }
                match endpoint.direction() {
                    UsbDirection::In => input = Some(endpoint.endpoint_number()),
                    UsbDirection::Out => {
                        output = Some((endpoint.endpoint_number(), endpoint.packet_size()));
                    }
                    _ => {}
                }
            }
            if let (Some(input), Some((output, output_packet))) = (input, output)
                && input != 0
                && output != 0
                && output_packet != 0
            {
                return Some(Endpoints {
                    configuration: configuration.configuration_value(),
                    interface: interface.interface_number(),
                    alternate: alternate.alternate_setting(),
                    input,
                    output,
                    output_packet,
                });
            }
        }
    }
    None
}

#[derive(Debug)]
struct WebUSBTransport {
    device: UsbDevice,
    endpoints: Endpoints,
    pending_setup: Option<Promise<js_sys::Undefined>>,
    owned: bool,
}

impl WebUSBTransport {
    async fn open(device: UsbDevice) -> Result<Self> {
        if device.opened() {
            return Err(protocol::invalid(
                "USB device is already open; disconnect its other client first",
            ));
        }
        let selected = device
            .configuration()
            .as_ref()
            .and_then(endpoints)
            .or_else(|| {
                device
                    .configurations()
                    .iter()
                    .find_map(|value| endpoints(&value.unchecked_into()))
            })
            .ok_or_else(|| protocol::invalid("No ADB bulk interface (ff/42/01) found"))?;
        let mut this = Self {
            device,
            endpoints: selected,
            pending_setup: None,
            owned: true,
        };
        let promise = this.device.open();
        this.setup(promise).await?;
        if this
            .device
            .configuration()
            .is_none_or(|c| c.configuration_value() != selected.configuration)
        {
            let promise = this.device.select_configuration(selected.configuration);
            this.setup(promise).await?;
        }
        let promise = this.device.claim_interface(selected.interface);
        this.setup(promise).await?;
        let promise = this
            .device
            .select_alternate_interface(selected.interface, selected.alternate);
        this.setup(promise).await?;
        Ok(this)
    }

    async fn setup(&mut self, promise: Promise<js_sys::Undefined>) -> Result<()> {
        self.pending_setup = Some(promise.clone());
        let result = JsFuture::from(promise).await;
        self.pending_setup = None;
        result.map_err(js_error)?;
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        JsFuture::from(self.device.close())
            .await
            .map_err(js_error)?;
        self.owned = false;
        Ok(())
    }

    async fn output(&mut self, bytes: &[u8]) -> Result<()> {
        let promise = self
            .device
            .transfer_out_with_u8_slice(self.endpoints.output, bytes)
            .map_err(js_error)?;
        let result = JsFuture::from(promise).await.map_err(js_error)?;
        if result.status() != UsbTransferStatus::Ok
            || result.bytes_written() as usize != bytes.len()
        {
            return Err(protocol::invalid("Incomplete or stalled WebUSB bulk write"));
        }
        Ok(())
    }
}

impl Drop for WebUSBTransport {
    fn drop(&mut self) {
        if !self.owned {
            return;
        }
        let device = self.device.clone();
        let pending = self.pending_setup.take();
        spawn_local(async move {
            // Setup methods change USB device state. If one was cancelled,
            // wait for it to settle before closing. For transferIn/Out, close
            // immediately so a pending read cannot prevent cleanup forever.
            if let Some(promise) = pending {
                let _ = JsFuture::from(promise).await;
            }
            let _ = JsFuture::from(device.close()).await;
        });
    }
}

impl Transport for WebUSBTransport {
    async fn read(&mut self, output: &mut [u8]) -> Result<usize> {
        let result = JsFuture::from(
            self.device
                .transfer_in(self.endpoints.input, u32::try_from(output.len())?),
        )
        .await
        .map_err(js_error)?;
        if result.status() != UsbTransferStatus::Ok {
            return Err(protocol::invalid("Stalled or failed WebUSB bulk read"));
        }
        let data = result
            .data()
            .ok_or_else(|| protocol::invalid("WebUSB read returned no data"))?;
        let length = data.byte_length();
        if length > output.len() {
            return Err(protocol::invalid("WebUSB read exceeded requested length"));
        }
        Uint8Array::new_with_byte_offset_and_length(
            &data.buffer(),
            u32::try_from(data.byte_offset())?,
            u32::try_from(length)?,
        )
        .copy_to(&mut output[..length]);
        Ok(length)
    }

    async fn write(&mut self, bytes: &[u8]) -> Result<()> {
        self.output(bytes).await?;
        if !bytes.is_empty()
            && bytes
                .len()
                .is_multiple_of(self.endpoints.output_packet as usize)
        {
            self.output(&[]).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
