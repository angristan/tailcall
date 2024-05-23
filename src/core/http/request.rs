use hyper::{Method, Uri, Version};
use anyhow::Result;
use derive_setters::Setters;
use http_body_util::BodyExt;
#[derive(Setters, Default)]
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub headers: http::HeaderMap,
    pub body: bytes::Bytes,
}

impl Request {
    pub async fn from_hyper(req: hyper::Request<hyper::Request<hyper::body::Incoming>>) -> Result<Self> {
        let (parts, body) = req.into_parts();
        let body = body.into_body().collect().await?.to_bytes();
        Ok(
            Request {
                method: parts.method,
                uri: parts.uri,
                version: parts.version,
                headers: parts.headers,
                body,
            }
        )
    }
}