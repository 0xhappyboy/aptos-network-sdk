use aptos_network_tool::address::address_to_bytes;
// src/contract.rs
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};

use crate::{
    AptosClient,
    trade::Trade,
    types::{
        ContractCall, ContractReadResult, ContractWriteResult, EntryFunctionPayload, ViewRequest,
    },
    wallet::Wallet,
};

/// default contract
pub const COIN_STORE: &str = "0x1::coin::CoinStore";
pub const APTOS_COIN: &str = "0x1::aptos_coin::AptosCoin";
pub const COIN: &str = "0x1::coin";
/// default contract function
pub const TRANSFER: &str = "transfer";
pub const BALANCE: &str = "balance";
pub const REGISTER: &str = "register";
pub const MINT: &str = "mint";
pub const BURN: &str = "burn";

pub struct Contract {}
impl Contract {
    /// read contract data (view read)
    pub async fn read(
        client: Arc<AptosClient>,
        contract_call: &ContractCall,
    ) -> Result<ContractReadResult, String> {
        let function = format!(
            "{}::{}::{}",
            contract_call.module_address, contract_call.module_name, contract_call.function_name
        );
        let view_request = ViewRequest {
            function,
            type_arguments: contract_call.type_arguments.clone(),
            arguments: contract_call.arguments.clone(),
        };
        match client.view(&view_request).await {
            Ok(result) => Ok(ContractReadResult {
                success: true,
                data: Value::Array(result),
                error: None,
            }),
            Err(e) => Ok(ContractReadResult {
                success: false,
                data: Value::Null,
                error: Some(e.to_string()),
            }),
        }
    }
    /// write contract
    pub async fn write(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        contract_call: ContractCall,
    ) -> Result<ContractWriteResult, String> {
        let function_str = format!(
            "{}::{}::{}",
            contract_call.module_address, contract_call.module_name, contract_call.function_name
        );
        let function_vec = function_str.as_bytes().to_vec();
        let mut type_args: Vec<Vec<u8>> = Vec::new();
        contract_call
            .type_arguments
            .iter()
            .for_each(|s| type_args.push(s.as_bytes().to_vec()));
        let mut args: Vec<Vec<u8>> = Vec::new();
        contract_call
            .arguments
            .iter()
            .for_each(|s| args.push(s.as_str().unwrap().to_string().as_bytes().to_vec()));
        let payload = EntryFunctionPayload {
            module_address: address_to_bytes(&contract_call.module_address)
                .unwrap()
                .to_vec(),
            module_name: address_to_bytes(&contract_call.module_name)
                .unwrap()
                .to_vec(),
            function_name: function_vec,
            type_arguments: type_args,
            arguments: args,
        };
        let raw_txn = Trade::create_call_contract_tx(
            Arc::clone(&client),
            Arc::clone(&wallet),
            None,
            30,
            2000,
            100,
            payload,
        )
        .await;
        // use wallet sign
        let signature = wallet.sign(&serde_json::to_vec(&raw_txn).unwrap()).unwrap();
        let signed_txn = json!({
            "transaction": raw_txn,
            "signature": {
                "type": "ed25519_signature",
                "public_key": wallet.public_key_hex()?,
                "signature": hex::encode(signature)
            }
        });
        match client.submit_transaction(&signed_txn).await {
            Ok(transaction) => {
                // awaiting
                if let Ok(confirmed_txn) = client.waiting_transaction(&transaction.hash, 30).await {
                    Ok(ContractWriteResult {
                        success: confirmed_txn.success,
                        transaction_hash: confirmed_txn.hash,
                        gas_used: confirmed_txn.max_gas_amount,
                        events: confirmed_txn
                            .events
                            .into_iter()
                            .map(|e| {
                                json!({
                                    "type": e.r#type,
                                    "data": e.data,
                                    "sequence_number": e.sequence_number
                                })
                            })
                            .collect(),
                        error: if confirmed_txn.success {
                            None
                        } else {
                            Some(confirmed_txn.vm_status)
                        },
                    })
                } else {
                    Ok(ContractWriteResult {
                        success: false,
                        transaction_hash: transaction.hash,
                        gas_used: "0".to_string(),
                        events: Vec::new(),
                        error: Some("Transaction confirmation timeout".to_string()),
                    })
                }
            }
            Err(e) => Ok(ContractWriteResult {
                success: false,
                transaction_hash: String::new(),
                gas_used: "0".to_string(),
                events: Vec::new(),
                error: Some(e.to_string()),
            }),
        }
    }
    /// batch read
    pub async fn batch_read(
        client: Arc<AptosClient>,
        calls: Vec<ContractCall>,
    ) -> Result<Vec<ContractReadResult>, String> {
        let mut results = Vec::new();
        for call in calls {
            results.push(Contract::read(Arc::clone(&client), &call).await.unwrap());
        }
        Ok(results)
    }
    /// listen contract events
    pub async fn listen_events(
        client: Arc<AptosClient>,
        address: &str,
        event_type: &str,
        callback: impl Fn(Result<Value, String>),
        interval_secs: u64,
    ) -> Result<(), ()> {
        let mut last_sequence_number: Option<u64> = None;
        loop {
            match client
                .get_account_event_vec(address, event_type, Some(100), None)
                .await
            {
                Ok(events) => {
                    for event in events {
                        if let Some(last_seq) = last_sequence_number {
                            let current_seq: u64 = event.sequence_number.parse().unwrap_or(0);
                            if current_seq > last_seq {
                                callback(Ok(event.data))
                            }
                        } else {
                            callback(Ok(event.data))
                        }
                        // update last sequence number
                        if let Ok(seq) = event.sequence_number.parse::<u64>() {
                            last_sequence_number = Some(seq);
                        }
                    }
                }
                Err(e) => callback(Err(format!("no event exists: {:?}", e).to_string())),
            }
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;
        }
    }
    
    /// get contract resource
    pub async fn get_contract_resource(
        client: Arc<AptosClient>,
        address: &str,
        resource_type: &str,
    ) -> Result<Option<Value>, String> {
        match client.get_account_resource(address, resource_type).await {
            Ok(resource) => match resource {
                Some(r) => Ok(Some(r.data)),
                None => Err(format!("get contract resource error: resource is none").to_string()),
            },
            Err(e) => Err(format!("get contract resource error: {:?}", e).to_string()),
        }
    }
}
