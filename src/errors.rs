#[derive(Debug)]
pub enum Error {
    Serde(serde_json::Error),
    Reqwest(reqwest::Error),
    Generic(String),
    UrlParse(url::ParseError),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Reqwest(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Serde(err)
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Error::UrlParse(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::Generic(err)
    }
}

impl<'a> From<&'a str> for Error {
    fn from(err: &str) -> Error {
        Error::Generic(err.into())
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
