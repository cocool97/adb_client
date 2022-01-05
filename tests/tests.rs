#[cfg(test)]
mod tests {
    use adb_client::AdbCommandProvider;
    use adb_client::AdbTcpConnexion;

    #[test]
    fn test_version() {
        let adb = AdbTcpConnexion::new();
        adb.version().unwrap();
    }

    #[test]
    fn test_devices() {
        let adb = AdbTcpConnexion::new();
        adb.devices().unwrap();
    }

    #[test]
    fn test_devices_long() {
        let adb = AdbTcpConnexion::new();
        adb.devices_long().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_wrong_addr() {
        let _ = AdbTcpConnexion::new().address("127.0.0.300").unwrap();
    }
}
