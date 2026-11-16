use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc,
};

use adb_client::ADBDeviceExt;

use crate::models::{ADBCliError, ADBCliResult, ForwardCommand, ReverseCommand};

/// Run a forward command. With `blocking`, for devices whose forwards live in this process, an added forward is
/// relayed until interrupted, then removed.
pub fn handle_forward_command(
    device: &dyn ADBDeviceExt,
    command: ForwardCommand,
    blocking: bool,
) -> ADBCliResult<()> {
    match command {
        ForwardCommand::RemoveAll => device.forward_remove_all()?,
        ForwardCommand::Remove { local } => device.forward_remove(&local)?,
        ForwardCommand::Add { local, remote } => {
            let local = device.forward(&remote, &local)?;
            log::info!("Forwarding host {local} to device {remote}");
            if blocking {
                wait_for_interruption()?;
                device.forward_remove(&local)?;
                log::info!("Forwarding stopped");
            }
        }
    }
    Ok(())
}

/// Run a reverse command. With `blocking`, for devices whose reverse forwards live in this process, an added reverse
/// forward is relayed until interrupted, then removed.
pub fn handle_reverse_command(
    device: &dyn ADBDeviceExt,
    command: ReverseCommand,
    blocking: bool,
) -> ADBCliResult<()> {
    match command {
        ReverseCommand::RemoveAll => device.reverse_remove_all()?,
        ReverseCommand::Remove { remote } => device.reverse_remove(&remote)?,
        ReverseCommand::Add { remote, local } => {
            device.reverse(&remote, &local)?;
            log::info!("Forwarding device {remote} to host {local}");
            if blocking {
                wait_for_interruption()?;
                device.reverse_remove(&remote)?;
                log::info!("Forwarding stopped");
            }
        }
    }
    Ok(())
}

/// Wait for Ctrl-C. A second one exits right away, e.g. if removing the forward hangs.
fn wait_for_interruption() -> ADBCliResult<()> {
    let (interrupted_sender, interrupted) = mpsc::channel();
    let interrupted_once = AtomicBool::new(false);
    ctrlc::set_handler(move || {
        if interrupted_once.swap(true, Ordering::SeqCst) {
            std::process::exit(130);
        }
        let _ = interrupted_sender.send(());
    })
    .map_err(|e| ADBCliError::Standard(Box::new(e)))?;

    log::info!("Press Ctrl-C to stop");
    let _ = interrupted.recv();
    Ok(())
}
