// TODO, see https://github.com/strake/joy.rs/blob/master/src/lib.rs

use vbus_core::Payload;

#[derive(bincode::Encode, bincode::Decode, Debug)]
pub struct JoystickData {
    // TODO
}

impl Payload for JoystickData {}

pub struct Joystick {}

impl Joystick {}

impl Drop for Joystick {
    fn drop(&mut self) {}
}
