use std::error::Error;
use std::str::FromStr;
use tokio;
use aries_askar::{EntryTag, PassKey, Store, StoreKeyMethod, TagFilter};
use aries_askar::postgres::{PostgresStore, PostgresStoreOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key_method = Some(StoreKeyMethod::default());
    let pass_key= PassKey::from("acapy");
    let profile = "";
    let store_opts = PostgresStoreOptions::new("{\"uri\":\"postgres://postgres:mysecretpassword@localhost:5432/askar_wallet\"}");

    let store = store_opts.unwrap().open(
        key_method,
        pass_key,
        Some(profile)
    ).await?;
    let mut session = store.session(Some(profile.to_string())).await.unwrap();

    let category = "category1";
    let name = "name1";
    let value = "value1";
    let tags = [EntryTag::Encrypted("tagkey1".to_string(),"tagval1".to_string())];
    session.insert(
        category,
        name,
        value.as_bytes(),
        Some(&tags),
        Some(0));

    let filter = TagFilter::from_str("").unwrap();
    let limit = 100;
    let for_update = false;
    let records = session.fetch_all(category, Some(filter), Some(limit), for_update);

    println!("Name: {}", name);
    // println!("Age: {}", age);

    Ok(())
}
