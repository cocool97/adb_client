#![cfg(unix)]
use std::os::unix::prelude::{AsRawFd, RawFd};

use termios::{tcsetattr, Termios, TCSANOW, VMIN, VTIME};

use crate::Result;

pub struct ADBTermios {
    fd: RawFd,
    old_termios: Termios,
    new_termios: Termios,
}

impl ADBTermios {
    pub fn new(fd: impl AsRawFd) -> Result<Self> {
        let mut new_termios = Termios::from_fd(fd.as_raw_fd())?;
        let old_termios = new_termios; // Saves previous state
        new_termios.c_lflag = 0;
        new_termios.c_cc[VTIME] = 0;
        new_termios.c_cc[VMIN] = 1;

        Ok(Self {
            fd: fd.as_raw_fd(),
            old_termios,
            new_termios,
        })
    }

    pub fn set_adb_termios(&mut self) -> Result<()> {
        Ok(tcsetattr(self.fd, TCSANOW, &self.new_termios)?)
    }
}

impl Drop for ADBTermios {
    fn drop(&mut self) {
        // Custom drop implementation, restores previous termios structure.
        tcsetattr(self.fd, TCSANOW, &self.old_termios).unwrap();
    }
}
