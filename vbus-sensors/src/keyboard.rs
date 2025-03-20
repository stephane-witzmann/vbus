use selecting::Selector;
use std::io::Read;
use std::os::fd::{AsRawFd, RawFd};
use std::thread::{JoinHandle, spawn};
use termios::*;
use vbus_core::tools::pipe_flag::*;
use vbus_core::{Channel, Payload};

#[derive(bincode::Encode, bincode::Decode, Debug)]
pub struct KeyboardData {
    pub data: char,
}

impl Payload for KeyboardData {}

pub struct Keyboard {
    termios_backup: Termios,
    flag_writer: PipeFlagWriter,
    join_handle: Option<JoinHandle<()>>,
}

impl Keyboard {
    pub fn new(channel: &Channel<KeyboardData>) -> Self {
        let old_termios = Termios::from_fd(stdin_raw_fd()).unwrap();
        let mut termios = old_termios; // copy

        termios.c_lflag &= !(ICANON | ECHO);
        termios.c_cc[VTIME] = 0;
        termios.c_cc[VMIN] = 1;
        termios_set(&termios);

        let (flag_reader, flag_writer) = pipe_flag();

        let channel_for_thread = channel.clone();

        let thread_join_handle = spawn(move || {
            let mut selector = Selector::new();
            selector.add_read(&flag_reader.as_raw_fd());

            let stdin_raw_fd = stdin_raw_fd();
            selector.add_read(&stdin_raw_fd);

            loop {
                let result = selector.select().unwrap();

                if result.is_read(&flag_reader.as_raw_fd()) {
                    return;
                }

                if !result.is_read(&stdin_raw_fd) {
                    continue;
                }

                let mut buffer = [0u8; 1];
                let result = std::io::stdin().read(&mut buffer).unwrap();
                if result != 1 {
                    continue;
                }

                channel_for_thread.push(KeyboardData {
                    data: buffer[0] as char,
                });
            }
        });

        Keyboard {
            termios_backup: old_termios,
            flag_writer,
            join_handle: Some(thread_join_handle),
        }
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        self.flag_writer.raise();
        self.join_handle.take().unwrap().join().unwrap();
        termios_set(&self.termios_backup);
    }
}

fn termios_set(termios: &Termios) {
    tcsetattr(stdin_raw_fd(), TCSANOW, termios).unwrap();
}

fn stdin_raw_fd() -> RawFd {
    std::io::stdin().as_raw_fd()
}

// TODO: test (using ospipe rather than stdin)
