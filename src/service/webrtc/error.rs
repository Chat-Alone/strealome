use webrtc::Error as WebRTCError;
use crate::controller::Response;
#[derive(Debug)]
pub struct Error(WebRTCError);

impl Error {
    pub fn custom(e: String) -> Self {
        WebRTCError::new(e).into()
    }
}

impl From<WebRTCError> for Error {
    fn from(e: WebRTCError) -> Self {
        Error(e)
    }
}

impl From<Error> for Response {
    fn from(e: Error) -> Self {
        Response::error(&e.0.to_string())
    }
}