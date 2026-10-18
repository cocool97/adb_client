#![cfg(any(target_os = "linux", target_os = "macos"))]

use std::os::unix::prelude::{AsRawFd, RawFd};

use termios::{TCSAFLUSH, Termios, VMIN, VTIME, cfmakeraw, tcsetattr};

use crate::models::{ADBCliError, ADBCliResult};

pub struct ADBTermios {
    fd: RawFd,
    old_termios: Termios,
    new_termios: Termios,
}

impl ADBTermios {
    pub fn new(fd: &impl AsRawFd) -> Result<Self, ADBCliError> {
        let mut current_termios = Termios::from_fd(fd.as_raw_fd())?;
        let saved_termios = current_termios; // Saves previous state for post-reapply

        // forward output byte-per-byte to the remote pty
        cfmakeraw(&mut current_termios);

        // no timeout but request at least one character per read.
        current_termios.c_cc[VTIME] = 0;
        current_termios.c_cc[VMIN] = 1;

        Ok(Self {
            fd: fd.as_raw_fd(),
            old_termios: saved_termios,
            new_termios: current_termios,
        })
    }

    pub fn set_adb_termios(&self) -> ADBCliResult<()> {
        Ok(tcsetattr(self.fd, TCSAFLUSH, &self.new_termios)?)
    }
}

impl Drop for ADBTermios {
    fn drop(&mut self) {
        // Custom drop implementation, restores previous termios structure.
        if let Err(e) = tcsetattr(self.fd, TCSAFLUSH, &self.old_termios) {
            log::error!("Error while dropping ADBTermios: {e}");
        }
    }
}
