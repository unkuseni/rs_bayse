//! Utility functions used across the client.

use rand::distr::{Alphanumeric, Distribution};
use rand::rng;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Build a URL-encoded query string from a parameter map.
///
/// Each key-value pair is appended as `key=value` with `&` separators.
/// Values are not URL-encoded (the API expects raw values).
///
/// # Returns
///
/// An empty string if the map is empty, otherwise a query string without
/// a leading `?`.
pub fn build_request<T: ToString>(parameters: &BTreeMap<String, T>) -> String {
    if parameters.is_empty() {
        return String::new();
    }

    let mut request = String::with_capacity(
        parameters
            .iter()
            .map(|(k, v)| k.len() + v.to_string().len() + 1)
            .sum(),
    );
    for (key, value) in parameters {
        request.push_str(key);
        request.push('=');
        let mut value_str = value.to_string();
        if value_str.starts_with('"') && value_str.ends_with('"') {
            value_str = value_str[1..value_str.len() - 1].to_string();
        }
        request.push_str(&value_str);
        request.push('&');
    }
    request.truncate(request.len() - 1);
    request
}

/// Serialise a parameter map to a JSON string for POST bodies.
///
/// Panics if serialisation fails. Prefer `serde_json::to_string` for
/// structured types.
pub fn build_json_request<T: Serialize>(parameters: &BTreeMap<String, T>) -> String {
    serde_json::to_string(parameters).expect("Failed to serialise parameters to JSON")
}

/// Convert a JSON value to `i64`.
pub fn to_i64(value: &Value) -> Option<i64> {
    value.as_i64()
}

/// Convert a JSON value to `u64`.
pub fn to_u64(value: &Value) -> Option<u64> {
    value.as_u64()
}

/// Current system timestamp in milliseconds since the Unix epoch.
///
/// Used to populate the `X-Timestamp` header for signed requests.
/// Panics if the system clock is set before the Unix epoch.
pub fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}

/// Generate a random alphanumeric string of the given length.
///
/// Useful for generating client-supplied identifiers (e.g.
/// `client_order_id`, `Idempotency-Key`).
///
/// # Example
///
/// ```
/// use bayse::util::generate_random_uid;
/// let uid = generate_random_uid(12);
/// assert_eq!(uid.len(), 12);
/// assert!(uid.chars().all(|c| c.is_ascii_alphanumeric()));
/// ```
pub fn generate_random_uid(length: usize) -> String {
    let mut uid = String::with_capacity(length);
    let mut rng = rng();
    for _ in 0..length {
        uid.push(Alphanumeric.sample(&mut rng) as char);
    }
    uid
}
