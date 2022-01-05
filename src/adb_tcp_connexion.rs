use std::{
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    str,
    str::FromStr,
};

use crate::{
    models::{AdbCommand, AdbRequestStatus, AdbVersion},
    AdbCommandProvider, Device, DeviceState, Result, RustADBError,
};

/// Represents an ADB-over-TCP connexion.
pub struct AdbTcpConnexion {
    socket_addr: SocketAddrV4,
    port: u16,
    address: Ipv4Addr,
}

impl AdbTcpConnexion {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self.socket_addr = SocketAddrV4::new(self.address, self.port);
        self
    }

    pub fn address<S: ToString>(mut self, address: S) -> Result<Self> {
        self.address = Ipv4Addr::from_str(&address.to_string())?;
        self.socket_addr = SocketAddrV4::new(self.address, self.port);
        Ok(self)
    }

    fn proxy_connexion(&self, adb_command: AdbCommand) -> Result<Vec<u8>> {
        let mut tcp_stream = TcpStream::connect(self.socket_addr)?;

        let adb_command_string = adb_command.to_string();

        let adb_request = format!(
            "{}{}",
            format!("{:04x}", adb_command_string.len()),
            adb_command_string
        );

        tcp_stream.write_all(adb_request.as_bytes())?;

        let mut request_status = [0; 4];
        tcp_stream.read_exact(&mut request_status)?;

        match AdbRequestStatus::from_str(str::from_utf8(&request_status.to_vec())?)? {
            AdbRequestStatus::Okay => {
                let mut length = [0; 4];
                tcp_stream.read_exact(&mut length)?;

                let u32_length = u32::from_str_radix(str::from_utf8(&length)?, 16)?;

                let mut body = vec![
                    0;
                    u32_length
                        .try_into()
                        .map_err(|_| RustADBError::ConvertionError)?
                ];
                if u32_length > 0 {
                    tcp_stream.read_exact(&mut body)?;
                }

                Ok(body)
            }
            AdbRequestStatus::Fail => Err(RustADBError::ADBRequestFailed),
        }
    }
}

impl Default for AdbTcpConnexion {
    fn default() -> Self {
        let default_port: u16 = 5037;
        let default_address = Ipv4Addr::new(127, 0, 0, 1);

        Self {
            socket_addr: SocketAddrV4::new(default_address, default_port),
            address: default_address,
            port: default_port,
        }
    }
}

impl AdbCommandProvider for AdbTcpConnexion {
    fn version(&self) -> Result<AdbVersion> {
        let version = self.proxy_connexion(AdbCommand::Version)?;

        Ok(AdbVersion::new(
            u32::from_str_radix(str::from_utf8(&version[0..2])?, 16)?,
            u32::from_str_radix(str::from_utf8(&version[2..4])?, 16)?,
        ))
    }

    fn devices(&self) -> Result<Vec<Device>> {
        let devices = self.proxy_connexion(AdbCommand::Devices)?;

        let mut vec_devices: Vec<Device> = vec![];
        for device in devices.split(|x| x.eq(&b'\n')) {
            if device.is_empty() {
                break;
            }

            let mut iter = device.split(|x| x.eq(&b'\t'));
            let identifier = iter.next().ok_or(RustADBError::IteratorError)?;
            let state = iter.next().ok_or(RustADBError::IteratorError)?;

            vec_devices.push(Device {
                identifier: String::from_utf8(identifier.to_vec())?,
                state: DeviceState::from_str(str::from_utf8(state)?)?,
            });
        }

        Ok(vec_devices)
    }

    fn devices_long(&self) -> Result<Vec<Device>> {
        let devices_long = self.proxy_connexion(AdbCommand::DevicesLong)?;

        // Split at '\n' (lines())
        // Split at '\t'
        // Identifier = [0]
        // Device state = [1]

        println!("Devices long: {:?}", std::str::from_utf8(&devices_long));

        Ok(vec![])
    }

    fn kill(&self) -> Result<()> {
        self.proxy_connexion(AdbCommand::Kill).map(|_| ())
    }

    // TODO: Change with Generator when feature stabilizes
    fn track_devices(&self, callback: fn(Device) -> Result<()>) -> Result<()> {
        let mut tcp_stream = TcpStream::connect(self.socket_addr)?;

        let adb_command_string = AdbCommand::TrackDevices.to_string();

        let adb_request = format!(
            "{}{}",
            format!("{:04x}", adb_command_string.len()),
            adb_command_string
        );

        tcp_stream.write_all(adb_request.as_bytes())?;

        let mut request_status = [0; 4];
        tcp_stream.read_exact(&mut request_status)?;

        match AdbRequestStatus::from_str(str::from_utf8(&request_status.to_vec())?)? {
            AdbRequestStatus::Okay => {
                loop {
                    // Reads first 4 bytes indicating payload length
                    let mut length = [0; 4];
                    tcp_stream.read_exact(&mut length)?;

                    let u32_length = u32::from_str_radix(str::from_utf8(&length)?, 16)?;

                    if u32_length > 0 {
                        let mut body = vec![
                            0;
                            u32_length
                                .try_into()
                                .map_err(|_| RustADBError::ConvertionError)?
                        ];
                        tcp_stream.read_exact(&mut body)?;

                        let mut iter = body.split(|x| x.eq(&b'\t'));
                        let identifier = iter.next().ok_or(RustADBError::IteratorError)?;
                        let state = iter.next().ok_or(RustADBError::IteratorError)?;

                        let device = Device {
                            identifier: String::from_utf8(identifier.to_vec())?,
                            state: DeviceState::from_str(
                                str::from_utf8(state)?.trim_matches('\n'),
                            )?,
                        };

                        callback(device)?;
                    }
                }
            }
            AdbRequestStatus::Fail => Err(RustADBError::ADBRequestFailed),
        }
    }
}
