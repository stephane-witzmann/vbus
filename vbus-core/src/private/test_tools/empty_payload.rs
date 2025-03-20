use crate::Payload;

#[derive(Default, bincode::Encode, bincode::Decode)]
pub(crate) struct EmptyPayload {}

impl Payload for EmptyPayload {}
