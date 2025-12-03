#[cfg(test)]
mod tests {
    use crate::JsonResult;
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
}
