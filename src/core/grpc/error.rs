use std::fmt::Display;

use derive_more::{DebugCustom, From};
use prost_reflect::DescriptorError;
use serde_json;

use crate::core::blueprint::GrpcMethod;

#[derive(From, DebugCustom)]
pub enum Error {
    #[debug(fmt = "Serde Json Error : {}", _0)]
    SerdeJsonError(serde_json::Error),

    #[debug(fmt = "Prost Encode Error")]
    ProstEncodeError(prost::EncodeError),

    #[debug(fmt = "Prost Decode Error")]
    ProstDecodeError(prost::DecodeError),

    #[debug(fmt = "Empty Response")]
    EmptyResponse,

    #[debug(fmt = "Couldn't resolve message")]
    MessageNotResolved,

    #[debug(fmt = "Descriptor pool error")]
    DescriptorPoolError(DescriptorError),

    #[debug(fmt = "Protox Parse Error")]
    ProtoxParseError(protox_parse::ParseError),

    #[debug(fmt = "Couldn't find method {}", ._0.name)]
    MissingMethod(GrpcMethod),

    #[debug(fmt = "Unable to find list field on type")]
    MissingListField,

    #[debug(fmt = "Field not found : {}", _0)]
    #[from(ignore)]
    MissingField(String),

    #[debug(fmt = "Couldn't find definitions for service {}", _0)]
    #[from(ignore)]
    MissingService(String),

    #[debug(fmt = "Failed to parse input according to type {}", _0)]
    #[from(ignore)]
    InputParsingFailed(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SerdeJsonError(e) => write!(f, "Serde Json Error: {}", e),
            Error::ProstEncodeError(_) => write!(f, "Prost Encode Error"),
            Error::ProstDecodeError(_) => write!(f, "Prost Decode Error"),
            Error::EmptyResponse => write!(f, "Empty Response"),
            Error::MessageNotResolved => write!(f, "Couldn't resolve message"),
            Error::DescriptorPoolError(_) => write!(f, "Descriptor pool error"),
            Error::ProtoxParseError(_) => write!(f, "Protox Parse Error"),
            Error::MissingMethod(method) => write!(f, "Couldn't find method {}", method.name),
            Error::MissingListField => write!(f, "Unable to find list field on type"),
            Error::MissingField(field) => write!(f, "Field not found : {}", field),
            Error::MissingService(service) => {
                write!(f, "Couldn't find definitions for service {}", service)
            }
            Error::InputParsingFailed(input_type) => {
                write!(f, "Failed to parse input according to type {}", input_type)
            }
        }
    }
}

pub type Result<A> = std::result::Result<A, Error>;
