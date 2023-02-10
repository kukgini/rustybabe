use std::str::FromStr;
use tokio;
use aries_askar::{PassKey, StoreKeyMethod, TagFilter};
use aries_askar::postgres::{PostgresStoreOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key_method = Some(StoreKeyMethod::default());
    let pass_key= PassKey::from("acapy");
    let profile = "askar-profile";
    let db_options = r#"postgres://postgres:mysecretpassword@localhost:5432/askar_wallet"#;
    let store_opts = PostgresStoreOptions::new(db_options);

    let store = store_opts.unwrap().open(key_method, pass_key, Some(profile)).await?;
    let mut session = store.session(Some(profile.to_string())).await.unwrap();

    let category = "connection";
    let filter_str = r###"
    "###;
    let filter = TagFilter::from_str(filter_str).unwrap();
    let limit = 100;
    let for_update = false;
    let _records = session.fetch_all(category, Some(filter), Some(limit), for_update).await?;

    // println!("Name: {}", name);
    // // println!("Age: {}", age);

    Ok(())
}
