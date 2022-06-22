use std::{string::FromUtf8Error, fmt};

use hyper::{Response, Body, StatusCode};
use tokio::io::{Error, ErrorKind};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum UnicomErrorKind{
    NotFound,
    Unknown,
    ParameterInvalid,
    InputInvalid,
    LostConnection,
    Internal,
    Timeout,
    DataInvalid,
    ParseError,
    Empty,
    ErrorUnknown,
    MandatoryMissing,
    NotAllowed,
    MethodNotAllowed,
    OutOfMemory,
    RenderFailed,
}

impl From<ErrorKind> for UnicomErrorKind{
    fn from(e: ErrorKind) -> Self{
        match e {
            ErrorKind::NotFound => UnicomErrorKind::NotFound,
            ErrorKind::PermissionDenied => UnicomErrorKind::NotAllowed,
            ErrorKind::ConnectionRefused => UnicomErrorKind::NotAllowed,
            ErrorKind::ConnectionReset => UnicomErrorKind::LostConnection,
            ErrorKind::ConnectionAborted => UnicomErrorKind::LostConnection,
            ErrorKind::NotConnected => UnicomErrorKind::LostConnection,
            ErrorKind::AddrInUse => UnicomErrorKind::NotAllowed,
            ErrorKind::AddrNotAvailable => UnicomErrorKind::LostConnection,
            ErrorKind::BrokenPipe => UnicomErrorKind::LostConnection,
            ErrorKind::InvalidInput => UnicomErrorKind::InputInvalid,
            ErrorKind::InvalidData => UnicomErrorKind::DataInvalid,
            ErrorKind::TimedOut => UnicomErrorKind::Timeout,
            ErrorKind::WriteZero => UnicomErrorKind::Empty,
            ErrorKind::Interrupted => UnicomErrorKind::LostConnection,
            ErrorKind::Unsupported => UnicomErrorKind::NotAllowed,
            ErrorKind::UnexpectedEof => UnicomErrorKind::ParseError,
            ErrorKind::OutOfMemory => UnicomErrorKind::OutOfMemory,
            _ => UnicomErrorKind::ErrorUnknown,
        }
    }
}

impl Into<StatusCode> for UnicomErrorKind{
    fn into(self) -> StatusCode {
        match self{
            UnicomErrorKind::NotFound => StatusCode::NOT_FOUND,
            UnicomErrorKind::Unknown => StatusCode::NO_CONTENT,
            UnicomErrorKind::ParameterInvalid => StatusCode::BAD_REQUEST,
            UnicomErrorKind::InputInvalid => StatusCode::NOT_ACCEPTABLE,
            UnicomErrorKind::LostConnection => StatusCode::FAILED_DEPENDENCY,
            UnicomErrorKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            UnicomErrorKind::Timeout => StatusCode::FAILED_DEPENDENCY,
            UnicomErrorKind::DataInvalid => StatusCode::EXPECTATION_FAILED,
            UnicomErrorKind::ParseError => StatusCode::INTERNAL_SERVER_ERROR,
            UnicomErrorKind::Empty => StatusCode::NO_CONTENT,
            UnicomErrorKind::ErrorUnknown => StatusCode::INTERNAL_SERVER_ERROR,
            UnicomErrorKind::MandatoryMissing => StatusCode::BAD_REQUEST,
            UnicomErrorKind::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            UnicomErrorKind::NotAllowed => StatusCode::FORBIDDEN,
            UnicomErrorKind::OutOfMemory => StatusCode::INTERNAL_SERVER_ERROR,
            UnicomErrorKind::RenderFailed => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UnicomError{
    kind: UnicomErrorKind,
    pub description: String,
}

impl UnicomError{
    pub fn new(kind: UnicomErrorKind, description: &str) -> UnicomError{
        UnicomError{
            kind,
            description: description.to_string(),
        }
    }

    pub fn from_utf8(message: Vec<u8>) -> Result<UnicomError, UnicomError>{
        Ok(serde_json::from_str(&String::from_utf8(message)?)?)
    }
}

impl fmt::Display for UnicomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<Error> for UnicomError{
    fn from(e: Error) -> Self{
        UnicomError::new(e.kind().into(), &e.to_string())
    }
}

impl From<serde_json::Error> for UnicomError{
    fn from(e: serde_json::Error) -> Self{
        UnicomError::new(UnicomErrorKind::ParseError, &format!("serde json error: {:?}",e))
    }
}

impl From<FromUtf8Error> for UnicomError{
    fn from(e: FromUtf8Error) -> Self {
        UnicomError::new(UnicomErrorKind::ParseError, &format!("Could not parse to String: {:?}",e))
    }
}

impl From<regex::Error> for UnicomError {
    fn from(e: regex::Error) -> Self {
        UnicomError::new(UnicomErrorKind::ParseError, &format!("Regex error: {:?}",e))
    }
}

impl From<tera::Error> for UnicomError{
    fn from(e: tera::Error) -> Self{
        UnicomError::new(UnicomErrorKind::RenderFailed, &format!("Render error: {:?}",e))
    }
}

impl From<toml::de::Error> for UnicomError{
    fn from(e: toml::de::Error) -> Self{
        UnicomError::new(UnicomErrorKind::ParseError, &format!("Config error: {:?}",e))
    }
}

impl Into<Response<Body>> for UnicomError{
    fn into(self) -> Response<Body> {
        let status: StatusCode = self.kind.into();
        Response::builder()
        .status(status)
        .body(Body::from(self.description)).unwrap()
    }
    
}