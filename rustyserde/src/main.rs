use std::fs;

pub mod parser;

fn main() {
    let result = parser::untyped_example();
    match result {
        Ok(result) => (),
        Err(error) => print!("{}", error),
    }
}
