#![allow(dead_code)]

mod joystick;
mod keyboard;
mod serial_settings;

pub use joystick::Joystick;
pub use keyboard::Keyboard;
pub use serial_settings::SerialSettings;

#[cfg(test)]
mod tests {
    #[test]
    fn dumb_test() {}
}
