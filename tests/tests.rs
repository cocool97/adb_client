#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::net::Ipv4Addr;
    use std::str::FromStr;

    use adb_client::AdbTcpConnection;
    use rand::Rng;

    fn new_client() -> AdbTcpConnection {
        let address = Ipv4Addr::from_str("127.0.0.1").unwrap();
        AdbTcpConnection::new(address, 5037).expect("Could not build ADB client...")
    }

    #[test]
    fn test_version() {
        let mut adb = new_client();
        adb.version().unwrap();
    }

    #[test]
    fn test_shell() {
        let mut adb = new_client();
        adb.shell_command(None, vec!["ls"]).unwrap();
        adb.shell_command(None, vec!["pwd"]).unwrap();
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
        let _ = AdbTcpConnection::new(address, 5037).expect("Could not create ADB connection...");
    }

    #[test]
    fn test_send_recv() {
        // Create random "Reader" in memory
        let mut key = [0u8; 1000];
        rand::thread_rng().fill(&mut key[..]);
        let mut c: Cursor<Vec<u8>> = Cursor::new(key.to_vec());

        let mut connection = new_client();

        const TEST_FILENAME: &'static str = "/data/local/tmp/test_file";
        // Send it
        connection
            .send::<&str, &str>(None, &mut c, TEST_FILENAME)
            .expect("cannot send file");

        // Pull it to memory
        let mut res = vec![];
        connection
            .recv::<&str, &str>(None, TEST_FILENAME, &mut res)
            .expect("cannot recv file");

        // diff
        assert_eq!(c.get_ref(), &res);

        connection
            .shell_command::<&str>(None, [format!("rm {TEST_FILENAME}").as_str()])
            .expect("cannot remove test file");
    }

    #[test]
    fn multiple_connexions() {
        let mut connection = new_client();

        for _ in 0..2 {
            let _ = connection.devices().expect("cannot get version");
        }
    }
}
