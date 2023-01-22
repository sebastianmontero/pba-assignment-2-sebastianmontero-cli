use std::fmt;

#[derive(Debug)]
pub struct Error {
    details: String,
}

impl Error {
    pub fn new(msg: &str) -> Error {
        Error {
            details: msg.to_string(),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.details
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl From<jsonrpsee_core::Error> for Error {
    fn from(err: jsonrpsee_core::Error) -> Self {
        Error::new(&format!("{:#?}", err))
    }
}

impl From<parity_scale_codec::Error> for Error {
    fn from(err: parity_scale_codec::Error) -> Self {
        Error::new(&format!("{:#?}", err))
    }
}

impl From<dotenv::Error> for Error {
    fn from(err: dotenv::Error) -> Self {
        Error::new(&format!("{:#?}", err))
    }
}

impl From<std::env::VarError> for Error {
    fn from(err: std::env::VarError) -> Self {
        Error::new(&format!("{:#?}", err))
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::new(&format!("{:#?}", err))
    }
}

impl From<sqlite::Error> for Error {
    fn from(err: sqlite::Error) -> Self {
        Error::new(&format!("{:#?}", err))
    }
}

impl From<sp_keystore::Error> for Error {
    fn from(err: sp_keystore::Error) -> Self {
        Error::new(&format!("{:#?}", err))
    }
}

impl From<sp_core::crypto::PublicError> for Error {
    fn from(err: sp_core::crypto::PublicError) -> Self {
        Error::new(&format!("{:#?}", err))
    }
}