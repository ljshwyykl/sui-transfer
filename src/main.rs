use chrono::Local;
use serde_json::json;

use std::io::Write;
use std::sync::Arc;
#[allow(unused_imports)]
use sui_sdk::rpc_types::{SuiEvent, SuiObjectInfo};
use sui_sdk::types::event::BalanceChangeType;

#[allow(unused_imports)]
use std::str::FromStr;

use std::time::Instant;
use std::{env, fs};
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore, Keystore};
use sui_sdk::types::messages::Transaction;
use sui_sdk::{
    json::SuiJsonValue,
    types::{
        base_types::ObjectID, base_types::SuiAddress, messages::ExecuteTransactionRequestType,
        SUI_FRAMEWORK_ADDRESS,
    },
    SuiClient,
};
use sui_types::crypto::SignatureScheme;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let now = Instant::now();

    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        panic!("args error")
    }

    let total = &args[3].to_string().parse::<usize>().unwrap();

    let args_arc = Arc::new(args);

    let mut handles = Vec::with_capacity(*total);
    for _i in 0..*total {
        handles.push(tokio::spawn(handle(args_arc.clone())));
    }
    // Wait for all of them to complete.
    for handle in handles {
        handle.await?;
    }

    /*
    for i in 0..*total {
        handle(phrase, object_id).await?;
        println!("{}", i)
    }

    */

    println!("总耗时：{} ms", now.elapsed().as_millis());

    Ok(())
}

async fn handle(args: Arc<Vec<String>>) -> Result<(), anyhow::Error> {
    let phrase_from = &args[1];
    let object_id = &args[2];

    let sui = SuiClient::new("https://fullnode.devnet.sui.io:443", None).await?;

    let temp_dir = env::temp_dir();
    let keystore_path = temp_dir.as_path().join("sui.keystore");
    let mut keystore = Keystore::from(FileBasedKeystore::new(&keystore_path).unwrap());
    let (address, phrase, _scheme) = keystore
        .generate_new_key(SignatureScheme::ED25519, None)
        .unwrap();
    // println!("address:{},phrase:{},scheme:{:?}", address, phrase, scheme);

    fs::create_dir_all("address").unwrap();

    let s1 = String::from("address/data");
    let s2 = String::from(".txt");
    let path = format!("{}{}{}", s1, Local::now().timestamp_millis(), s2);
    let mut file = std::fs::File::create(path).expect("create failed");
    file.write_all(phrase.as_bytes()).expect("write failed");
    file.write_all("\n".as_bytes()).expect("write failed");
    file.write_all(address.to_string().as_bytes())
        .expect("write failed");
    // println!("data written to file");

    let mut keystore2 = Keystore::from(FileBasedKeystore::new(&keystore_path).unwrap());

    let from_address = keystore2
        .import_from_mnemonic(&phrase_from, SignatureScheme::ED25519, None)
        .unwrap();

    // println!("from_address:{}", from_address);

    //let gas_object_id = get_first_object_id(from_address, &sui).await?;
    let gas_object_id = ObjectID::from_str(&object_id)?;
    // println!("gas_object_id:{}", gas_object_id);

    // let gas_object_id = object_id;

    let recipient = address;

    // println!("recipient:{}", recipient);
    // let transfer_tx = sui
    //     .transaction_builder()
    //     .transfer_sui(from_address, gas_object_id, 1000, recipient, Some(30000))
    //     .await?;

    // Create a sui transfer transaction
    let transfer_tx = sui
        .transaction_builder()
        .transfer_sui(from_address, gas_object_id, 1000, recipient, Some(30000))
        .await?;

    let signature = keystore.sign(&from_address, &transfer_tx.to_bytes())?;

    // Execute the transaction
    let response = sui
        .quorum_driver()
        .execute_transaction(
            Transaction::new(transfer_tx, signature).verify()?,
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;

    // println!("{:#?}", response);
    // let signature = keystore2.sign(&from_address, &transfer_tx)?;

    // // Execute the transaction
    // let response = sui
    //     .quorum_driver()
    //     .execute_transaction(Transaction::from_data(transfer_tx, signature))
    //     .await?;
    // // println!("transfser {:#?}", response);
    // let events: Vec<SuiEvent::CoinBalanceChange> = response.effects.unwrap().events;

    let events = response.effects.unwrap().events;

    let events1 = events
        .iter()
        .filter(|s| match s {
            SuiEvent::CoinBalanceChange {
                package_id: _,
                transaction_module: _,
                sender: _,
                change_type,
                owner: _,
                coin_type: _,
                coin_object_id: _,
                version: _,
                amount: _,
            } => change_type == &BalanceChangeType::Receive,

            _ => {
                return false;
            }
        })
        .collect::<Vec<_>>();
    // println!("events1 {:#?}", events1);

    if let SuiEvent::CoinBalanceChange {
        package_id: _,
        transaction_module: _,
        sender: _,
        change_type: _,
        owner: _,
        coin_type: _,
        coin_object_id,
        version: _,
        amount: _,
    } = events1[0]
    {
        // println!("{}", coin_object_id);

        create_nft(
            recipient,
            ObjectID::from(SUI_FRAMEWORK_ADDRESS),
            "devnet_nft",
            "mint",
            Some(*coin_object_id),
            10000,
            keystore,
            &sui,
        )
        .await?;
    }

    Ok(())
}

async fn create_nft(
    my_address: SuiAddress,
    package: ObjectID,
    module: &str,
    function: &str,
    gas: Option<ObjectID>,
    gas_budget: u64,
    keystore: Keystore,
    sui: &SuiClient,
) -> Result<(), anyhow::Error> {
    // let sui = SuiClient::new_rpc_client("https://fullnode.devnet.sui.io:443", None).await?;

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

    let signature = keystore.sign(&my_address, &transfer_tx.to_bytes())?;

    // Execute the transaction
    sui.quorum_driver()
        .execute_transaction(
            Transaction::new(transfer_tx, signature).verify()?,
            Some(ExecuteTransactionRequestType::WaitForEffectsCert),
        )
        .await?;

    // println!("{:?}", transaction_response);

    // let nft_id = effects
    //     .created
    //     .first()
    //     .ok_or_else(|| anyhow!("Failed to create NFT"))?
    //     .reference
    //     .object_id;

    // // println!("{:?}", transaction_response);
    Ok(())
}

/*
async fn get_first_object_id(
    address: SuiAddress,
    sui: &SuiClient,
) -> Result<ObjectID, anyhow::Error> {
    // let sui = SuiClient::new_rpc_client("https://fullnode.devnet.sui.io:443", None).await?;
    // let address = SuiAddress::from_str("0x004230a90f543a4993ea3b15954be615f14a71b3")?;
    let object_refs = sui.read_api().get_objects_owned_by_address(address).await?;

    // let v: Value = serde_json::from_str(objects)?;
    let object_id = object_refs
        .into_iter()
        .filter(|s| s.type_ == "0x2::coin::Coin<0x2::sui::SUI>")
        .collect::<Vec<SuiObjectInfo>>()
        .first()
        .unwrap()
        .object_id;

    // println!("{}", object_id);
    // // println!("{:?}", objects);
    Ok(object_id)
}

*/
#[tokio::test]

async fn test_get_first_object_id() -> Result<(), anyhow::Error> {
    let sui = SuiClient::new("https://fullnode.devnet.sui.io:443", None).await?;
    let address = SuiAddress::from_str("0x004230a90f543a4993ea3b15954be615f14a71b3")?;
    let object_refs = sui.read_api().get_objects_owned_by_address(address).await?;

    let o2: Vec<SuiObjectInfo> = object_refs
        .into_iter()
        .filter(|s| s.type_ == "0x2::coin::Coin<0x2::sui::SUI>")
        .collect();

    // println!("o2 {:#?}", o2);
    assert_eq!(3, 3);

    Ok(())
}
