mod register;
mod error;

#[macro_export]
macro_rules! unwrap {
    ($result:expr) => {
        match $result {
            Ok(v) => v,
            Err(e) => return ResponseError::from(e).into_response(),
        }
    };
}