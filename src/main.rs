use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
struct MyTest;

#[derive(Serialize)]
struct MyError;

fn test() -> Result<MyTest, MyError> {
    Ok(MyTest)
}

fn main() {
    let e: Value = serde_json::to_value(test()).unwrap();

    println!("Hello, world! {:?}", e);
}
