mod register;
mod error;
pub mod response;

use std::net::SocketAddr;
pub use error::Error;
pub use response::Response;

#[macro_export]
macro_rules! unwrap {
    ($result:expr) => {
        match $result {
            Ok(v) => v,
            Err(e) => return Error::from(e).into_response(),
        }
    };
}

pub async fn listen(addr: SocketAddr) {
    // let app = register::router();
    todo!()
}

