use std::sync::mpsc;

use adb_client::{
    ADBDeviceExt,
    mdns::{MDNSDevice, MDNSDiscoveryService},
    usb::ADBUSBDevice,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting mDNS device discovery...");

    // Create a channel to receive discovered devices information
    // let (sender, receiver) = mpsc::channel::<MDNSDevice>();

    // // Create and start the discovery service
    // let mut discovery = MDNSDiscoveryService::new()?;
    // discovery.start(sender)?;

    // loop {
    //     let device = receiver.recv()?;
    //     println!("{device}");
    // }

    let mut device = ADBUSBDevice::autodetect().unwrap();
    device.framebuffer(&"/tmp/image1.png").unwrap();
    device.framebuffer(&"/tmp/image2.png").unwrap();

    Ok(())
}
