use reqwest;
use serde_json;


#[derive(Debug)]
pub enum Error {
    SerdeError(serde_json::Error),
    ReqwestError(reqwest::Error),
    Generic(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::ReqwestError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::SerdeError(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::Generic(err)
    }
}

/*
impl RustixError {
    #[allow(dead_code)]
    pub fn to_string(self) -> String {
        use self::RustixError::*;

        match self {
            Generic(s) => String::from(s),
            Reqwest(e) => e.to_string(),
        }
    }
}
*/
