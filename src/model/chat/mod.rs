use std::sync::atomic::AtomicI32;

mod message;
mod author;
mod signal;

pub use signal::{Signal, Event, Message, Payload};
pub use author::Author;
// pub use message::{Message, MessagePayload};

static ID: AtomicI32  = AtomicI32::new(0);
pub fn gen_id() -> i32 {
    ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

