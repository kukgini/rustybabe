use serde_json::{Result, Value};
use std::error::Error;
use std::io;

pub fn untyped_example() -> Result<()> {
    let json = serde_json::from_reader(io::stdin());
    let v: Value = json.unwrap();
    if let Some(results) = v["results"].as_array() {
        println!("id");
        for result in results {
            println!("{}", result["id"].as_str().unwrap());
        }
    }
    Ok(())
}