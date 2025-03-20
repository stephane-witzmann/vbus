use super::CameraData;
use vbus_core::Channel;

pub struct Uvc {
    // TODO
}

impl Uvc {
    pub fn new(_channel: &Channel<CameraData>) -> Self {
        // TODO
        Uvc {}
    }
}

impl Drop for Uvc {
    fn drop(&mut self) {
        // TODO [RAII]
    }
}
