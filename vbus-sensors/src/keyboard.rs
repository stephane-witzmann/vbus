// TOOD

use std::os::fd::{AsRawFd, RawFd};
use termios::*;

pub struct Keyboard {
    termios_backup: Termios,
}

impl Keyboard {
    fn new() -> Self {
        let old_termios = Termios::from_fd(stdin_raw_fd()).unwrap();
        let mut termios = old_termios; // copy

        termios.c_lflag &= !(ICANON | ECHO);
        termios.c_cc[VTIME] = 0;
        termios.c_cc[VMIN] = 1;

        termios_set(&termios);

        Keyboard {
            termios_backup: old_termios,
        }
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        termios_set(&self.termios_backup);
    }
}

fn termios_set(termios: &Termios) {
    tcsetattr(stdin_raw_fd(), TCSANOW, termios).unwrap();
}

fn stdin_raw_fd() -> RawFd {
    std::io::stdin().as_raw_fd()
}
