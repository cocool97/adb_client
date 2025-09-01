use std::sync::mpsc;
use std::time::Duration;

use adb_client::mdns::{MDNSDevice, MDNSDiscoveryService};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting mDNS device discovery...");

    // Create a channel to receive discovered devices information
    let (sender, receiver) = mpsc::channel::<MDNSDevice>();

    // Create and start the discovery service
    let mut discovery = MDNSDiscoveryService::new()?;
    discovery.start(sender)?;

    loop {
        if let Ok(device) = receiver.recv_timeout(Duration::from_millis(100)) {
            println!("Found device: {}", device.fullname);
            println!("  Addresses: {:?}", device.addresses);
        }
    }
}
