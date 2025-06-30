use std::env;
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

#[test]
fn test() {
    let payload = Payload::Ping(signal::Ping::new(114514, 6));
    let signal = Signal::new(1919810, Author::User(6666), payload);
    use serde_json::json;
    println!("{}", json!(signal));
    
    let parsed_signal = serde_json::from_str::<Signal>(json!(signal).to_string().as_str()).unwrap();
    println!("{}", json!(parsed_signal));
    
    use env;
    println!( "{:?}", env::var("PYTHON"))
}