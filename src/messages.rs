use crate::{logger, types::RequestMessage};

pub fn initialize(message: &RequestMessage) -> Option<String> {
    logger::log(&message.params, logger::MessageType::Error);
    None
}
