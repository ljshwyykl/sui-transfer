use chrono::Local;
use std::{env, fs};
use std::io::Write;
use std::str::FromStr;
use sui_sdk::crypto::{AccountKeystore, FileBasedKeystore, Keystore};
use sui_sdk::types::crypto::SignatureScheme;
use sui_sdk::types::messages::Transaction;
use sui_sdk::{
    types::{base_types::ObjectID, messages::ExecuteTransactionRequestType},
    SuiClient,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().collect();

    let phrase = &args[1];
    let object_id = &args[2];

    let total = &args[3].to_string().parse::<usize>().unwrap();

    for _i in 0..*total {
        handle(phrase, object_id).await?;
    }

    Ok(())
}

async fn handle(phrase_from: &str, object_id: &str) -> Result<(), anyhow::Error> {
    let sui = SuiClient::new_rpc_client("https://fullnode.devnet.sui.io:443", None).await?;

    let temp_dir = env::temp_dir();
    let keystore_path = temp_dir.as_path().join("sui.keystore");
    let mut keystore = Keystore::from(FileBasedKeystore::new(&keystore_path).unwrap());
    let (address, phrase, scheme) = keystore
        .generate_new_key(SignatureScheme::ED25519, None)
        .unwrap();
    println!("address:{},phrase:{},scheme:{:?}", address, phrase, scheme);

    fs::create_dir_all("address").unwrap();
    
    let s1 = String::from("address/data");
    let s2 = String::from(".txt");
    let path = format!("{}{}{}", s1, Local::now().timestamp_millis(), s2);
    let mut file = std::fs::File::create(path).expect("create failed");
    file.write_all(phrase.as_bytes()).expect("write failed");
    file.write_all("\n".as_bytes()).expect("write failed");
    file.write_all(address.to_string().as_bytes())
        .expect("write failed");
    println!("data written to file");

    let mut keystore2 = Keystore::from(FileBasedKeystore::new(&keystore_path).unwrap());

    let my_address = keystore2
        .import_from_mnemonic(&phrase_from, SignatureScheme::ED25519, None)
        .unwrap();

    let gas_object_id = ObjectID::from_str(&object_id)?;
    let recipient = address;

    let transfer_tx = sui
        .transaction_builder()
        .transfer_sui(my_address, gas_object_id, 1000, recipient, Some(10))
        .await?;

    let signature = keystore2.sign(&my_address, &transfer_tx.to_bytes())?;

    // Execute the transaction
    let transaction_response = sui
        .quorum_driver()
        .execute_transaction(
            Transaction::new(transfer_tx, signature),
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;

    println!("{:?}", transaction_response);
    Ok(())
}
