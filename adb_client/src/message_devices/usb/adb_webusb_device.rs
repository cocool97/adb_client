use std::{
    io::{Read, Write},
    path::Path,
};

use crate::{
    ADBDeviceExt, Result,
    usb::{WebUsbTransport, adb_usb_device::ADBUSBDevice},
};

/// Implement Android USB device reachable over wired USB
#[derive(Debug)]
pub struct ADBWebUsbDevice {
    inner: ADBUSBDevice<WebUsbTransport>,
}

impl ADBWebUsbDevice {
    /// Instantiate a new [`ADBRusb`]
    pub fn new(_vendor_id: u16, _product_id: u16) -> Result<Self> {
        todo!()
    }
}

impl ADBDeviceExt for ADBWebUsbDevice {
    #[inline]
    fn shell_command(&mut self, command: &[&str], output: &mut dyn Write) -> Result<()> {
        self.inner.shell_command(command, output)
    }

    #[inline]
    fn shell<'a>(&mut self, reader: &mut dyn Read, writer: Box<dyn Write + Send>) -> Result<()> {
        self.inner.shell(reader, writer)
    }

    #[inline]
    fn stat(&mut self, remote_path: &str) -> Result<crate::AdbStatResponse> {
        self.inner.stat(remote_path)
    }

    #[inline]
    fn pull(&mut self, source: &dyn AsRef<str>, output: &mut dyn Write) -> Result<()> {
        self.inner.pull(source, output)
    }

    #[inline]
    fn push(&mut self, stream: &mut dyn Read, path: &dyn AsRef<str>) -> Result<()> {
        self.inner.push(stream, path)
    }

    #[inline]
    fn reboot(&mut self, reboot_type: crate::RebootType) -> Result<()> {
        self.inner.reboot(reboot_type)
    }

    #[inline]
    fn install(&mut self, apk_path: &dyn AsRef<Path>) -> Result<()> {
        self.inner.install(apk_path)
    }

    #[inline]
    fn uninstall(&mut self, package: &str) -> Result<()> {
        self.inner.uninstall(package)
    }

    #[inline]
    fn framebuffer_inner(&mut self) -> Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>> {
        self.inner.framebuffer_inner()
    }
}
