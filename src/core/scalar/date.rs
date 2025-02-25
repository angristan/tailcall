use chrono::DateTime;
use schemars::schema::Schema;
use schemars::{schema_for, JsonSchema};
use tailcall_macros::ScalarDefinition;

use crate::core::json::JsonLikeOwned;

/// A date string, such as 2007-12-03, is compliant with the full-date format outlined in section 5.6 of the RFC 3339 (https://datatracker.ietf.org/doc/html/rfc3339) profile of the ISO 8601 standard for the representation of dates and times using the Gregorian calendar.
#[derive(JsonSchema, Default, ScalarDefinition)]
pub struct Date {
    #[allow(dead_code)]
    #[serde(rename = "Date")]
    pub date: String,
}

impl super::Scalar for Date {
    /// Function used to validate the date
    fn validate<Value: JsonLikeOwned>(&self) -> fn(&Value) -> bool {
        |value| {
            if let Some(date_str) = value.as_str() {
                if DateTime::parse_from_rfc3339(date_str).is_ok() {
                    return true;
                }
            }
            false
        }
    }

    fn schema(&self) -> Schema {
        Schema::Object(schema_for!(Self).schema)
    }
}

#[cfg(test)]
mod test {
    use async_graphql_value::ConstValue;

    use super::*;
    use crate::core::scalar::Scalar;

    #[test]
    fn test_date() {
        let date = Date::default();
        let validate = date.validate()(&ConstValue::String("2020-01-01T12:00:00Z".to_string()));
        assert!(validate);
    }

    #[test]
    fn test_invalid_date() {
        let date = Date::default();
        let validate = date.validate()(&ConstValue::String("2023-03-08T12:45:26".to_string()));
        assert!(!validate);
    }

    #[test]
    fn test_invalid_value() {
        let date = Date::default();
        let validate = date.validate()(&ConstValue::Null);
        assert!(!validate);
    }
}
