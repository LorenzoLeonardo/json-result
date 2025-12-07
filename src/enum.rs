use serde::de::{DeserializeOwned, Error};

/// A generic enum representing a JSON result that can either be a success (`Ok`) with a value of type `T`
/// or an error (`Err`) with a value of type `E`.
///
/// This enum is designed to be serialized and deserialized using Serde's untagged enum representation,
/// allowing it to seamlessly handle JSON values that could match either type.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum JsonResult<T, E> {
    /// Variant representing a successful result containing a value of type `T`.
    Ok(T),
    /// Variant representing an error result containing a value of type `E`.
    Err(E),
}

impl<T, E> From<JsonResult<T, E>> for serde_json::Value
where
    T: serde::Serialize,
    E: serde::Serialize,
{
    /// Converts a `JsonResult<T, E>` into a `serde_json::Value`.
    ///
    /// Serializes the contained value in either the `Ok` or `Err` variant into a JSON value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use serde_json::json;
    /// # use json_result::r#enum::JsonResult;
    /// let res: JsonResult<i32, String> = JsonResult::Ok(42);
    /// let json_value: serde_json::Value = res.into();
    /// assert_eq!(json_value, json!(42));
    /// ```
    fn from(res: JsonResult<T, E>) -> Self {
        match res {
            JsonResult::Ok(v) => serde_json::json!(v),
            JsonResult::Err(e) => serde_json::json!(e),
        }
    }
}

impl<T, E> TryFrom<serde_json::Value> for JsonResult<T, E>
where
    T: DeserializeOwned,
    E: DeserializeOwned,
{
    type Error = serde_json::Error;

    /// Attempts to convert a `serde_json::Value` into a `JsonResult<T, E>` by
    /// trying to deserialize it first into `T` (success variant), then into `E` (error variant).
    ///
    /// If deserialization into both types fails, returns a combined error message detailing both failures.
    ///
    /// # Errors
    ///
    /// Returns a `serde_json::Error` if the input JSON value cannot be parsed as either `T` or `E`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use serde_json::json;
    /// # use json_result::r#enum::JsonResult;
    /// let json_val = json!(42);
    /// let res: JsonResult<i32, String> = json_val.try_into().unwrap();
    /// match res {
    ///     JsonResult::Ok(val) => assert_eq!(val, 42),
    ///     JsonResult::Err(_) => panic!("Expected Ok variant"),
    /// }
    /// ```
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let t_res = serde_json::from_value::<T>(value.clone());
        let e_res = serde_json::from_value::<E>(value);

        match (t_res, e_res) {
            (Ok(v), _) => Ok(JsonResult::Ok(v)),
            (_, Ok(e)) => Ok(JsonResult::Err(e)),
            (Err(t_err), Err(e_err)) => {
                let t_name = std::any::type_name::<T>();
                let e_name = std::any::type_name::<E>();
                let message = format!(
                    "Failed to parse as {}: {}\nFailed to parse as {}: {}",
                    t_name, t_err, e_name, e_err
                );
                Err(serde_json::Error::custom(message))
            }
        }
    }
}
