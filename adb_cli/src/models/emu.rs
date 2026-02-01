use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct EmulatorCommand {
    #[clap(short = 's', long = "serial")]
    pub serial: String,
    #[clap(subcommand)]
    pub command: EmuCommand,
}

#[derive(Debug, Subcommand)]
pub enum EmuCommand {
    /// Send a SMS with given phone number and given content
    Sms {
        phone_number: String,
        content: String,
    },
    /// Rotate device screen from 90°
    Rotate,
    /// Get the AVD discovery path of this emulator
    AvdDiscoveryPath,
    /// Get the gRPC control protocol port of this emulator
    AvdGrpcPort,
    /// Send a raw console command to the emulator
    Raw {
        /// The raw console command to send
        command: String,
    },
}
