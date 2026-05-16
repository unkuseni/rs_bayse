//! Serde helper functions for custom (de)serialisation.
//!
//! Provides custom serde modules for handling string-encoded numeric fields
//! commonly returned by the API.

use serde::{de, Deserialize, Deserializer, Serializer};
use std::str::FromStr;

/// Custom deserialization module for handling strings as optional `f64`.
///
/// Used for fields like `avg_price` or `leverage` that may be empty or absent
/// in API responses. Empty strings are treated as `None`.
pub mod string_to_float_optional {
    use super::*;

    /// Serializes an `Option<f64>` as a string.
    pub fn serialize<S>(value: &Option<f64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(v) => serializer.serialize_str(&v.to_string()),
            None => serializer.serialize_str(""),
        }
    }

    /// Deserializes a string to an `Option<f64>`, returning `None` for empty strings.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s) if s.trim().is_empty() => Ok(None),
            Some(s) => f64::from_str(&s).map(Some).map_err(de::Error::custom),
            None => Ok(None),
        }
    }
}

/// Module for serializing and deserializing optional u64 values as strings.
pub mod string_to_u64_optional {
    use super::*;

    /// Serializes an Option<u64> as a string, using an empty string for None.
    pub fn serialize<S>(value: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(v) => serializer.serialize_str(&v.to_string()),
            None => serializer.serialize_str(""),
        }
    }

    /// Deserializes a string to an Option<u64>, returning None for empty strings.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s) if s.trim().is_empty() => Ok(None),
            Some(s) => u64::from_str(&s).map(Some).map_err(de::Error::custom),
            None => Ok(None),
        }
    }
}

/// Deserializes a string to a u64, handling empty strings and invalid formats.
pub mod string_to_u64 {
    use super::*;

    /// Serialize a u64 as a string.
    pub fn serialize<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = value.to_string();
        serializer.serialize_str(&s)
    }

    /// Deserialize a string to a u64.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<u64>().map_err(de::Error::custom)
    }
}

/// Deserializes a string to a u32, handling empty strings and invalid formats.
pub mod string_to_u32 {
    use super::*;

    /// Serialize a u32 as a string.
    pub fn serialize<S>(value: &u32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = value.to_string();
        serializer.serialize_str(&s)
    }

    /// Deserialize a string to a u32.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<u32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<u32>().map_err(de::Error::custom)
    }
}

/// Deserializes a string to an f64, handling empty strings and invalid formats.
pub mod string_to_float {
    use super::*;

    /// Serialize an f64 as a string.
    pub fn serialize<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = value.to_string();
        serializer.serialize_str(&s)
    }

    /// Deserialize a string as an f64.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<f64>().map_err(de::Error::custom)
    }
}

/// Checks if an optional string is empty or None.
pub fn is_empty_or_none(s: &Option<String>) -> bool {
    s.as_ref().is_none_or(|s| s.is_empty())
}

/// Custom deserialization function to treat empty strings as `None`.
pub fn empty_string_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(ref s) if s.trim().is_empty() => Ok(None),
        Some(s) => {
            let kind = T::deserialize(serde_json::Value::String(s))
                .map(Some)
                .map_err(de::Error::custom)?;
            Ok(kind)
        }
        None => Ok(None),
    }
}
