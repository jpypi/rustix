use reqwest::Error as ReqwestErr;

#[derive(Debug)]
pub enum RustixError {
    Generic(String),
    Reqwest(ReqwestErr)
}

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
