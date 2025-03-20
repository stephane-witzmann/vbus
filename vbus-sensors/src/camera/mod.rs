use vbus_core::Payload;

pub mod uvc;

#[derive(bincode::Encode, bincode::Decode, Debug)]
pub struct CameraData {
    // TODO
}

impl Payload for CameraData {}
