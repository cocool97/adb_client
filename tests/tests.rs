#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use std::str::FromStr;

    use adb_client::AdbTcpConnexion;

    fn new_client() -> AdbTcpConnexion {
        let address = Ipv4Addr::from_str("127.0.0.1").unwrap();
        AdbTcpConnexion::new(address, 5037).expect("Could not build ADB client...")
    }

    #[test]
    fn test_version() {
        let mut adb = new_client();
        adb.version().unwrap();
    }

    #[test]
    fn test_devices() {
        let mut adb = new_client();
        adb.devices().unwrap();
    }

    #[test]
    fn test_devices_long() {
        let mut adb = new_client();
        adb.devices_long().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_wrong_addr() {
        let address = Ipv4Addr::from_str("127.0.0.300").unwrap();
        let _ = AdbTcpConnexion::new(address, 5037).expect("Could not create ADB connexion...");
    }
}
