use reqwest;
use std::error::Error;
use std::io;
use std::process;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let api_key = "api-key";
    let api_token = "api-token";
    let client = reqwest::Client::new();
    let mut rdr = csv::Reader::from_reader(io::stdin());
    for result in rdr.records() {
        let record = result?;
        let id = record.get(0).unwrap();
        let url = format!("https://some.url/{}",id);
        let result = client.delete(&url)
            .header("X-API-KEY", api_key)
            .header("Authorization", format!("Bearer {}", api_token))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .send()
            .await
            .unwrap();
        match result.status() {
            reqwest::StatusCode::OK => {
                println!("O: {}", url);
            },
            reqwest::StatusCode::NOT_FOUND => {
                println!("X: {}", url)
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                println!("F: Need to grab a new token");
            },
            _ => {
                println!("E: {:?}", result);
            },
        }
    }
    Ok(())
}
