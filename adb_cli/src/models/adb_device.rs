use adb_client::{
    ADBDeviceExt, server_device::ADBServerDevice, tcp::ADBTcpDevice, usb::ADBUSBDevice,
};
use image::{ImageBuffer, Rgba};

/// Wrapper around the various ADB device types.
/// Implements [`ADBDeviceExt`] to provide common device operations.
/// Truly missing a macro to automatically forward calls to the inner device.
pub enum ADBDevice {
    Server(ADBServerDevice),
    Usb(ADBUSBDevice),
    Tcp(ADBTcpDevice),
}

impl ADBDeviceExt for ADBDevice {
    fn shell_command<W: std::io::Write>(
        &mut self,
        command: &str,
        stdout: Option<&mut W>,
        stderr: Option<&mut W>,
    ) -> adb_client::Result<Option<u8>> {
        match self {
            Self::Server(device) => device.shell_command(command, stdout, stderr),
            Self::Usb(device) => device.shell_command(command, stdout, stderr),
            Self::Tcp(device) => device.shell_command(command, stdout, stderr),
        }
    }

    fn shell<R: std::io::Read, W: std::io::Write + Send>(
        &mut self,
        reader: &mut R,
        writer: W,
    ) -> adb_client::Result<()> {
        match self {
            Self::Server(device) => device.shell(reader, writer),
            Self::Usb(device) => device.shell(reader, writer),
            Self::Tcp(device) => device.shell(reader, writer),
        }
    }

    fn exec<R: std::io::Read, W: std::io::Write + Send>(
        &mut self,
        command: &str,
        reader: &mut R,
        writer: W,
    ) -> adb_client::Result<()> {
        match self {
            Self::Server(device) => device.exec(command, reader, writer),
            Self::Usb(device) => device.exec(command, reader, writer),
            Self::Tcp(device) => device.exec(command, reader, writer),
        }
    }

    fn stat<P: AsRef<std::path::Path>>(
        &mut self,
        remote_path: P,
    ) -> adb_client::Result<adb_client::AdbStatResponse> {
        match self {
            Self::Server(device) => device.stat(remote_path),
            Self::Usb(device) => device.stat(remote_path),
            Self::Tcp(device) => device.stat(remote_path),
        }
    }

    fn pull<P: AsRef<std::path::Path>, W: std::io::Write>(
        &mut self,
        source: P,
        output: &mut W,
    ) -> adb_client::Result<()> {
        match self {
            Self::Server(device) => device.pull(source, output),
            Self::Usb(device) => device.pull(source, output),
            Self::Tcp(device) => device.pull(source, output),
        }
    }

    fn push<R: std::io::Read, P: AsRef<std::path::Path>>(
        &mut self,
        stream: &mut R,
        path: P,
    ) -> adb_client::Result<()> {
        match self {
            Self::Server(device) => device.push(stream, path),
            Self::Usb(device) => device.push(stream, path),
            Self::Tcp(device) => device.push(stream, path),
        }
    }

    fn list<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> adb_client::Result<Vec<adb_client::ADBListItemType>> {
        match self {
            Self::Server(device) => device.list(path),
            Self::Usb(device) => device.list(path),
            Self::Tcp(device) => device.list(path),
        }
    }

    fn reboot(&mut self, reboot_type: adb_client::RebootType) -> adb_client::Result<()> {
        match self {
            Self::Server(device) => device.reboot(reboot_type),
            Self::Usb(device) => device.reboot(reboot_type),
            Self::Tcp(device) => device.reboot(reboot_type),
        }
    }

    fn remount(&mut self) -> adb_client::Result<Vec<adb_client::RemountInfo>> {
        match self {
            Self::Server(device) => device.remount(),
            Self::Usb(device) => device.remount(),
            Self::Tcp(device) => device.remount(),
        }
    }

    fn root(&mut self) -> adb_client::Result<()> {
        match self {
            Self::Server(device) => device.root(),
            Self::Usb(device) => device.root(),
            Self::Tcp(device) => device.root(),
        }
    }

    fn install<P: AsRef<std::path::Path>>(
        &mut self,
        apk_path: P,
        user: Option<&str>,
    ) -> adb_client::Result<()> {
        match self {
            Self::Server(device) => device.install(apk_path, user),
            Self::Usb(device) => device.install(apk_path, user),
            Self::Tcp(device) => device.install(apk_path, user),
        }
    }

    fn uninstall(&mut self, package: &str, user: Option<&str>) -> adb_client::Result<()> {
        match self {
            Self::Server(device) => device.uninstall(package, user),
            Self::Usb(device) => device.uninstall(package, user),
            Self::Tcp(device) => device.uninstall(package, user),
        }
    }

    fn enable_verity(&mut self) -> adb_client::Result<()> {
        match self {
            Self::Server(device) => device.enable_verity(),
            Self::Usb(device) => device.enable_verity(),
            Self::Tcp(device) => device.enable_verity(),
        }
    }

    fn disable_verity(&mut self) -> adb_client::Result<()> {
        match self {
            Self::Server(device) => device.disable_verity(),
            Self::Usb(device) => device.disable_verity(),
            Self::Tcp(device) => device.disable_verity(),
        }
    }

    fn framebuffer_inner(&mut self) -> adb_client::Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        match self {
            Self::Server(device) => device.framebuffer_inner(),
            Self::Usb(device) => device.framebuffer_inner(),
            Self::Tcp(device) => device.framebuffer_inner(),
        }
    }
}
