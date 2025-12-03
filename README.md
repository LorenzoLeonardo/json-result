# JsonResult

`JsonResult` is a Rust generic enum designed for seamlessly handling JSON values that can represent either a success (`Ok`) or error (`Err`) result, with flexible types. It integrates tightly with [Serde](https://serde.rs/) for serialization and deserialization.

[![Latest Version](https://img.shields.io/crates/v/json-result.svg)](https://crates.io/crates/json-result)
[![License](https://img.shields.io/github/license/LorenzoLeonardo/json-result.svg)](LICENSE-MIT)
[![Documentation](https://docs.rs/json-result/badge.svg)](https://docs.rs/json-result)
[![Build Status](https://github.com/LorenzoLeonardo/json-result/workflows/Rust/badge.svg)](https://github.com/LorenzoLeonardo/json-result/actions)
[![Downloads](https://img.shields.io/crates/d/json-result)](https://crates.io/crates/json-result)

## Features

- Supports untagged enum representation for natural JSON parsing.
- Converts to and from `serde_json::Value` easily.
- Provides detailed error messages when deserialization fails.
- Generic over success (`T`) and error (`E`) types.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Usage
```rust
use serde_json::json;
use std::convert::TryFrom;

let success_json = json!({"id": 1, "name": "Alice"});
let error_json = json!({"error_code": 404, "message": "Not Found"});

// Define your success and error types
#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct User {
    id: u32,
    name: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct ApiError {
    error_code: u16,
    message: String,
}

type MyJsonResult = JsonResult<User, ApiError>;

// Deserialize JSON into JsonResult
let res_ok: MyJsonResult = success_json.try_into().unwrap();
let res_err: MyJsonResult = error_json.try_into().unwrap();

match res_ok {
    JsonResult::Ok(user) => println!("User: {:?}", user),
    JsonResult::Err(e) => println!("Error: {:?}", e),
}

match res_err {
    JsonResult::Ok(user) => println!("User: {:?}", user),
    JsonResult::Err(e) => println!("Error: {:?}", e),
}

// Convert JsonResult back to serde_json::Value
let json_val: serde_json::Value = res_ok.into();
println!("Serialized back to JSON: {}", json_val);
```