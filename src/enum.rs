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

impl<T, E> From<Result<T, E>> for JsonResult<T, E> {
    fn from(r: Result<T, E>) -> Self {
        match r {
            Ok(v) => JsonResult::Ok(v),
            Err(e) => JsonResult::Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::r#enum::JsonResult;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct GoodT {
        x: u32,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct BadE {
        msg: String,
    }

    #[test]
    fn test_good_case_ok_t() {
        let original: JsonResult<GoodT, BadE> = JsonResult::Ok(GoodT { x: 123 });

        let json = serde_json::to_value(&original).unwrap();
        let parsed = JsonResult::<GoodT, BadE>::try_from(json).unwrap();

        match parsed {
            JsonResult::Ok(v) => assert_eq!(v, GoodT { x: 123 }),
            _ => panic!("Expected Ok(T)"),
        }
    }

    #[test]
    fn test_good_case_err_e() {
        let original: JsonResult<GoodT, BadE> = JsonResult::Err(BadE { msg: "fail".into() });

        let json = serde_json::to_value(&original).unwrap();
        let parsed = JsonResult::<GoodT, BadE>::try_from(json).unwrap();

        match parsed {
            JsonResult::Err(e) => assert_eq!(e, BadE { msg: "fail".into() }),
            _ => panic!("Expected Err(E)"),
        }
    }

    #[test]
    fn test_bad_case_neither_matches() {
        let json = serde_json::json!({ "something": 9999 });

        let result = JsonResult::<GoodT, BadE>::try_from(json);

        assert!(result.is_err());

        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("GoodT"));
        assert!(msg.contains("BadE"));
        assert!(msg.contains("Failed to parse"));
    }

    #[test]
    fn test_round_trip_t() {
        let original: JsonResult<GoodT, BadE> = JsonResult::Ok(GoodT { x: 42 });

        let json: serde_json::Value = original.into();
        let parsed = JsonResult::<GoodT, BadE>::try_from(json).unwrap();

        match parsed {
            JsonResult::Ok(v) => assert_eq!(v.x, 42),
            _ => panic!("Round trip for T failed"),
        }
    }

    #[test]
    fn test_round_trip_e() {
        let original: JsonResult<GoodT, BadE> = JsonResult::Err(BadE { msg: "boom".into() });

        let json: serde_json::Value = original.into();
        let parsed = JsonResult::<GoodT, BadE>::try_from(json).unwrap();

        match parsed {
            JsonResult::Err(v) => assert_eq!(v.msg, "boom"),
            _ => panic!("Round trip for E failed"),
        }
    }

    #[test]
    fn test_empty_object() {
        let json = serde_json::json!({});

        // Neither GoodT nor BadE should parse successfully from empty object
        let result = JsonResult::<GoodT, BadE>::try_from(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_null_value() {
        let json = serde_json::json!(null);

        let result = JsonResult::<GoodT, BadE>::try_from(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_primitive_value_matches_t() {
        // If GoodT was just a number, primitive JSON number should parse as T
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct NumberT(u32);

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct StringE(String);

        let json = serde_json::json!(123u32);
        let parsed = JsonResult::<NumberT, StringE>::try_from(json).unwrap();

        match parsed {
            JsonResult::Ok(NumberT(n)) => assert_eq!(n, 123),
            _ => panic!("Expected Ok(NumberT)"),
        }
    }

    #[test]
    fn test_primitive_value_matches_e() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct NumberT(u32);

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct StringE(String);

        let json = serde_json::json!("error message");
        let parsed = JsonResult::<NumberT, StringE>::try_from(json).unwrap();

        match parsed {
            JsonResult::Err(StringE(s)) => assert_eq!(s, "error message"),
            _ => panic!("Expected Err(StringE)"),
        }
    }

    #[test]
    fn test_ambiguous_value() {
        // A JSON value that could deserialize to both T and E if they have same structure
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Ambiguous {
            value: u32,
        }

        let json = serde_json::json!({ "value": 55 });

        // Because we try T first, expect Ok variant
        let parsed = JsonResult::<Ambiguous, Ambiguous>::try_from(json).unwrap();

        match parsed {
            JsonResult::Ok(Ambiguous { value }) => assert_eq!(value, 55),
            _ => panic!("Expected Ok variant for ambiguous type"),
        }
    }

    #[test]
    fn test_deeply_nested_json() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct NestedT {
            nested: Option<Box<NestedT>>,
            val: u32,
        }

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct NestedE {
            error: String,
        }

        let json = serde_json::json!({
            "nested": {
                "nested": null,
                "val": 10
            },
            "val": 5
        });

        let parsed = JsonResult::<NestedT, NestedE>::try_from(json).unwrap();

        match parsed {
            JsonResult::Ok(n) => {
                assert_eq!(n.val, 5);
                assert!(n.nested.is_some());
                let inner = n.nested.unwrap();
                assert_eq!(inner.val, 10);
                assert!(inner.nested.is_none());
            }
            _ => panic!("Expected Ok with nested structure"),
        }
    }

    #[test]
    fn test_invalid_json_structure() {
        // JSON array will not deserialize to GoodT or BadE structs
        let json = serde_json::json!([1, 2, 3]);

        let result = JsonResult::<GoodT, BadE>::try_from(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_message_contains_correct_type_names() {
        // This triggers error with complex type names to ensure message includes them
        #[derive(Debug, Serialize, Deserialize)]
        struct ComplexType {
            field: String,
        }

        let json = serde_json::json!("just a string");

        let result = JsonResult::<ComplexType, ComplexType>::try_from(json);
        assert!(result.is_err());

        let err_str = result.unwrap_err().to_string();
        assert!(err_str.contains("ComplexType"));
        assert!(err_str.contains("Failed to parse"));
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct OKData {
        value: i32,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct ErrData {
        message: String,
    }

    #[test]
    fn test_from_result_ok() {
        let r: Result<OKData, ErrData> = Ok(OKData { value: 123 });

        let jr: JsonResult<OKData, ErrData> = JsonResult::from(r);

        match jr {
            JsonResult::Ok(ok) => assert_eq!(ok.value, 123),
            _ => panic!("Expected JsonResult::Ok"),
        }
    }

    #[test]
    fn test_from_result_err() {
        let r: Result<OKData, ErrData> = Err(ErrData {
            message: "boom".into(),
        });

        let jr: JsonResult<OKData, ErrData> = JsonResult::from(r);

        match jr {
            JsonResult::Err(e) => assert_eq!(e.message, "boom"),
            _ => panic!("Expected JsonResult::Err"),
        }
    }

    #[test]
    fn test_from_result_type_check() {
        // Just ensures this compiles & converts correctly
        let res: Result<i32, &str> = Ok(10);
        let jr: JsonResult<i32, &str> = res.into();

        assert!(matches!(jr, JsonResult::Ok(10)));
    }

    #[test]
    fn test_from_result_error_type_check() {
        let res: Result<i32, &str> = Err("wrong");
        let jr: JsonResult<i32, &str> = res.into();

        assert!(matches!(jr, JsonResult::Err("wrong")));
    }
}
