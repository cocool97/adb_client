use std::sync::mpsc;

use adb_client::mdns::{MDNSDevice, MDNSDiscoveryService};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting mDNS device discovery...");

    // Create a channel to receive discovered devices information
    let (sender, receiver) = mpsc::channel::<MDNSDevice>();

    // Create and start the discovery service
    let mut discovery = MDNSDiscoveryService::new()?;
    discovery.start(sender)?;

    loop {
        let device = receiver.recv()?;
        println!("{device}");
    }
}
