mod directive;
mod endpoint;
mod endpoint_set;
mod operation;
mod partial_request;
mod path;
mod query_params;
mod type_map;
mod typed_variables;

pub use endpoint_set::{Checked, EndpointSet, Unchecked};

use crate::core::http::RequestBody;

type Request = hyper::Request<RequestBody>;
