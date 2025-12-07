use std::ops::{Deref, DerefMut};

use serde::de::{DeserializeOwned, Error as DeError};
use serde::{Deserialize, Serialize};

/// JsonResult<T, E>
///
/// A small serde-compatible wrapper that serializes either the Ok(T) value or the Err(E) value,
/// and deserializes by attempting to parse the payload as `T` first and `E` second.
///
/// This is useful when a response payload may be either a success object or an error object
/// and you want to round-trip that semantics with serde.
///
/// # Examples
///
/// Serialize an Ok value:
/// ```rust
/// use json_result::r#struct::JsonResult;
/// let jr = JsonResult::<i32, &str>(Ok(100));
/// let s = serde_json::to_string(&jr).unwrap();
/// assert_eq!(s, "100");
/// ```
///
/// Deserialize into JsonResult:
/// ```rust
/// use json_result::r#struct::JsonResult;
/// let jr: JsonResult<i32, String> = serde_json::from_str("42").unwrap();
/// match jr.0 {
///     Ok(v) => assert_eq!(v, 42),
///     Err(_) => panic!("expected Ok"),
/// }
/// ```
#[derive(Debug)]
pub struct JsonResult<T, E>(pub Result<T, E>);

impl<T, E> Serialize for JsonResult<T, E>
where
    T: Serialize,
    E: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.0 {
            Ok(v) => v.serialize(serializer),
            Err(e) => e.serialize(serializer),
        }
    }
}

impl<'de, T, E> Deserialize<'de> for JsonResult<T, E>
where
    T: DeserializeOwned,
    E: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // First try to deserialize as T (Ok)
        let value = serde_json::Value::deserialize(deserializer)?;

        let try_t: Result<T, _> = serde_json::from_value(value.clone());
        let try_e: Result<E, _> = serde_json::from_value(value.clone());

        match (try_t, try_e) {
            (Ok(v), _) => Ok(JsonResult(Ok(v))),
            (_, Ok(e)) => Ok(JsonResult(Err(e))),
            (Err(t_err), Err(e_err)) => {
                let t_name = std::any::type_name::<T>();
                let e_name = std::any::type_name::<E>();

                let msg = format!(
                    "Failed to parse as {}: {}\nFailed to parse as {}: {}",
                    t_name, t_err, e_name, e_err
                );

                Err(DeError::custom(msg))
            }
        }
    }
}

impl<T, E> From<JsonResult<T, E>> for serde_json::Value
where
    T: Serialize,
    E: Serialize,
{
    fn from(value: JsonResult<T, E>) -> Self {
        match value.0 {
            Ok(v) => serde_json::json!(v),
            Err(e) => serde_json::json!(e),
        }
    }
}

// Deref to Result<T, E>
impl<T, E> Deref for JsonResult<T, E> {
    type Target = Result<T, E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, E> DerefMut for JsonResult<T, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use std::ops::DerefMut;

    use super::JsonResult;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct GoodT {
        v: u32,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct BadE {
        msg: String,
    }

    #[test]
    fn test_ok_serialization() {
        let jr = JsonResult::<i32, String>(Ok(100));
        let s = serde_json::to_string(&jr).unwrap();
        assert_eq!(s, "100");
    }

    #[test]
    fn test_err_serialization() {
        let jr = JsonResult::<i32, &str>(Err("boom"));
        let s = serde_json::to_string(&jr).unwrap();
        assert_eq!(s, "\"boom\"");
    }

    #[test]
    fn test_deserialize_ok() {
        let json = "42";
        let jr: JsonResult<i32, String> = serde_json::from_str(json).unwrap();
        assert_eq!(jr.0, Ok(42));
    }

    #[test]
    fn test_deserialize_err() {
        let json = "\"error occurred\"";
        let jr: JsonResult<i32, String> = serde_json::from_str(json).unwrap();
        assert_eq!(jr.0, Err("error occurred".to_string()));
    }

    #[test]
    fn test_struct_ok_case() {
        let json = serde_json::json!({ "v": 123 });
        let jr: JsonResult<GoodT, BadE> = serde_json::from_value(json).unwrap();

        assert_eq!(jr.0, Ok(GoodT { v: 123 }));
    }

    #[test]
    fn test_struct_err_case() {
        let json = serde_json::json!({ "msg": "fail" });
        let jr: JsonResult<GoodT, BadE> = serde_json::from_value(json).unwrap();

        assert_eq!(jr.0, Err(BadE { msg: "fail".into() }));
    }

    #[test]
    fn test_round_trip_ok() {
        let original = JsonResult::<GoodT, BadE>(Ok(GoodT { v: 55 }));
        let json = serde_json::to_value(&original).unwrap();
        let parsed: JsonResult<GoodT, BadE> = serde_json::from_value(json).unwrap();

        assert_eq!(parsed.0, Ok(GoodT { v: 55 }));
    }

    #[test]
    fn test_round_trip_err() {
        let original = JsonResult::<GoodT, BadE>(Err(BadE { msg: "x".into() }));
        let json = serde_json::to_value(&original).unwrap();
        let parsed: JsonResult<GoodT, BadE> = serde_json::from_value(json).unwrap();

        assert_eq!(parsed.0, Err(BadE { msg: "x".into() }));
    }

    #[test]
    fn test_invalid_json_fails() {
        let json = serde_json::json!([1, 2, 3]); // neither T nor E matches
        let result = serde_json::from_value::<JsonResult<GoodT, BadE>>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_message_contains_type_names() {
        #[derive(Debug, Serialize, Deserialize)]
        struct Complex {
            a: String,
        }

        let json = serde_json::json!(12345);

        let result = serde_json::from_value::<JsonResult<Complex, Complex>>(json);
        assert!(result.is_err());

        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Complex"));
        assert!(msg.contains("Failed to parse"));
    }

    #[test]
    fn test_ambiguous_matches_t_first() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Amb {
            x: u32,
        }

        let json = serde_json::json!({ "x": 10 });

        let jr: JsonResult<Amb, Amb> = serde_json::from_value(json).unwrap();

        // Ok should win because T is tried first
        assert_eq!(jr.0, Ok(Amb { x: 10 }));
    }

    #[test]
    fn test_null_fails() {
        let json = serde_json::json!(null);
        let result = serde_json::from_value::<JsonResult<GoodT, BadE>>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_value_into_serde_value_ok() {
        let jr = JsonResult::<i32, &str>(Ok(7));
        let v: serde_json::Value = jr.into();
        assert_eq!(v, serde_json::json!(7));
    }

    #[test]
    fn test_value_into_serde_value_err() {
        let jr = JsonResult::<i32, &str>(Err("oops"));
        let v: serde_json::Value = jr.into();
        assert_eq!(v, serde_json::json!("oops"));
    }

    #[test]
    fn deref_allows_calling_result_methods() {
        let jr = JsonResult::<u32, &'static str>(Ok(42));

        // Calling Result methods directly on JsonResult
        assert!(jr.is_ok());
        assert_eq!(jr.as_ref().unwrap(), &42);
    }

    #[test]
    fn deref_works_with_pattern_matching() {
        let jr = JsonResult::<u32, &'static str>(Ok(7));

        // Should behave like a Result in patterns
        match *jr {
            Ok(v) => assert_eq!(v, 7),
            Err(_) => panic!("Expected Ok"),
        }
    }

    #[test]
    fn deref_mut_allows_modifying_result() {
        let mut jr = JsonResult::<u32, &'static str>(Ok(10));

        // Modify the inner value via DerefMut
        if let Ok(v) = jr.deref_mut() {
            *v = 99;
        }

        assert_eq!(jr.unwrap(), 99);
    }

    #[test]
    fn deref_mut_replaces_result() {
        let mut jr = JsonResult::<u32, &'static str>(Ok(1));

        // Replace whole Result<T,E>
        *jr = Err("failed");

        assert!(jr.is_err());
        assert_eq!(jr.unwrap_err(), "failed");
    }

    #[test]
    fn can_be_used_like_result_in_functions() {
        fn take_result(r: &Result<u32, &'static str>) -> u32 {
            *r.as_ref().unwrap()
        }

        let jr = JsonResult(Ok(55));

        // JsonResult derefs to Result
        assert_eq!(take_result(&jr), 55);
    }
}
