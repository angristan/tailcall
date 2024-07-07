use std::fmt::Display;

use derive_more::{DebugCustom, From};
use reqwest::header::InvalidHeaderValue;

#[derive(From, DebugCustom)]
pub enum Error {
    #[debug(fmt = "Reqwest Error")]
    Reqwest(reqwest::Error),

    #[debug(fmt = "Invalid Header Value")]
    InvalidHeaderValue(InvalidHeaderValue),

    #[debug(fmt = "Serde JSON Error")]
    SerdeJson(serde_json::Error),

    #[debug(fmt = "Url Parser Error")]
    UrlParser(url::ParseError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Reqwest(_) => write!(f, "Reqwest Error"),
            Error::InvalidHeaderValue(_) => write!(f, "Invalid Header Value"),
            Error::SerdeJson(_) => write!(f, "Serde JSON Error"),
            Error::UrlParser(_) => write!(f, "Url Parser Error"),
        }
    }
}

pub type Result<A> = std::result::Result<A, Error>;
