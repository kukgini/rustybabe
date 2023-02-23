use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut r = csv::Reader::from_path("data/data.csv")?;
    for result in r.records() {
        let record = result?;
        let first = record.get(0).unwrap();
        println!("{}", first);
    }
    Ok(())
}
