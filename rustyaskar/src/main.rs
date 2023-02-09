use tokio;
use aries_askar::Store;
use aries_askar::storage::options::IntoOptions;
use aries_askar::postgres::{PostgresStore, PostgresStoreOptions};

#[tokio::main]
async fn main() {
    let url = match std::env::var("POSTGRES_URL") {
        Ok(p) if !p.is_empty() => p,
        _ => "postgres://postgres:mysecretpassword@localhost:5432/acapy".to_string(),
    };
    let spec_uri = url.into_options().unwrap();
    let key_method = "kdf:argon2i";
    let pass_key="acapy";
    let profile = "";
    let store = spec_uri.open_backend(
        key_method,
        pass_key,
        profile.as_ref().map(String::as_str)
    ).await?;

    store.insert("name".to_string(), "John".to_string());
    store.insert("age".to_string(), "30".to_string());

    let name = store.get("name").unwrap();
    let age = store.get("age").unwrap();

    println!("Name: {}", name);
    println!("Age: {}", age);
}
