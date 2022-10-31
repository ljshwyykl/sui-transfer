use chrono::Local;
use serde_json::json;

use std::io::Write;

use std::{env, fs};
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore, Keystore};
use sui_sdk::types::crypto::SignatureScheme;
use sui_sdk::types::messages::Transaction;
use sui_sdk::{
    json::SuiJsonValue,
    types::{
        base_types::ObjectID, base_types::SuiAddress, messages::ExecuteTransactionRequestType,
        SUI_FRAMEWORK_ADDRESS,
    },
    SuiClient,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().collect();

    println!("{}", args.len());
    if args.len() != 3 {
        panic!("args error")
    }

    let phrase = &args[1];
    // let object_id = &args[2];

    let total = &args[2].to_string().parse::<usize>().unwrap();

    for _i in 0..*total {
        handle(phrase).await?;
    }

    Ok(())
}

async fn handle(phrase_from: &str) -> Result<(), anyhow::Error> {
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

    let gas_object_id = get_first_object_id(my_address).await?;
    println!("my_address:{}", gas_object_id);

    // let gas_object_id = object_id;

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
            Transaction::new(transfer_tx, signature).verify()?,
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;

    println!("{:?}", transaction_response);

    let gas_object_id_2 = get_first_object_id(recipient).await?;
    println!("recipient address:{}", gas_object_id_2);

    create_nft(
        my_address,
        ObjectID::from(SUI_FRAMEWORK_ADDRESS),
        "devnet_nft",
        "mint",
        Some(gas_object_id_2),
        1000,
        keystore2,
    )
    .await?;

    Ok(())
}

async fn create_nft(
    my_address: SuiAddress,
    package: ObjectID,
    module: &str,
    function: &str,
    gas: Option<ObjectID>,
    gas_budget: u64,
    keystore2: Keystore,
) -> Result<(), anyhow::Error> {
    let sui = SuiClient::new_rpc_client("https://fullnode.devnet.sui.io:443", None).await?;

    let args_json = json!([
        "Qknow NFT",
        "An NFT created by the Sui Command Line Tool",
        "ipfs://bafkreibngqhl3gaa7daob4i2vccziay2jjlp435cf66vhono7nrvww53ty"
    ]);
    let mut args = vec![];
    for a in args_json.as_array().unwrap() {
        args.push(SuiJsonValue::new(a.clone()).unwrap());
    }

    let transfer_tx = sui
        .transaction_builder()
        .move_call(
            my_address,
            package,
            module,
            function,
            vec![],
            args,
            gas,
            gas_budget,
        )
        .await?;

    let signature = keystore2.sign(&my_address, &transfer_tx.to_bytes())?;

    // Execute the transaction
    let transaction_response = sui
        .quorum_driver()
        .execute_transaction(
            Transaction::new(transfer_tx, signature).verify()?,
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;

    println!("{:?}", transaction_response);
    // let nft_id = effects
    //     .created
    //     .first()
    //     .ok_or_else(|| anyhow!("Failed to create NFT"))?
    //     .reference
    //     .object_id;

    // println!("{:?}", transaction_response);
    Ok(())
}

async fn get_first_object_id(address: SuiAddress) -> Result<ObjectID, anyhow::Error> {
    let sui = SuiClient::new_rpc_client("https://fullnode.devnet.sui.io:443", None).await?;
    // let address = SuiAddress::from_str("0x004230a90f543a4993ea3b15954be615f14a71b3")?;
    let object_refs = sui.read_api().get_objects_owned_by_address(address).await?;

    // let v: Value = serde_json::from_str(objects)?;
    let object_id = object_refs.first().unwrap().object_id;
    println!("{}", object_id);
    // println!("{:?}", objects);
    Ok(object_id)
}
