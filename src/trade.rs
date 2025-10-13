use crate::{AptosClient, types::EntryFunctionPayload, wallet::Wallet};
use aptos_network_tool::{address::address_to_bytes, signature::serialize_transaction_and_sign};
use serde_json::{Value, json};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Trade;

impl Trade {
    /// build transfer info
    pub async fn create_transfer_tx(
        client: Arc<AptosClient>,
        sender: Arc<Wallet>,
        recipient: &str,
        amount: u64,
        sequence_number: Option<u64>,
        expiration_secs: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
    ) -> Result<Value, String> {
        let sequence_number = match sequence_number {
            Some(seq) => seq,
            None => {
                let account_info = client
                    .get_account_info(&sender.address().unwrap())
                    .await
                    .unwrap();
                account_info.sequence_number.parse().unwrap()
            }
        };
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expiration_timestamp = current_timestamp + expiration_secs;
        // build transaction payload
        let payload = json!({
            "type": "entry_function_payload",
            "function": "0x1::coin::transfer",
            "type_arguments": ["0x1::aptos_coin::AptosCoin"],
            "arguments": [recipient, amount.to_string()]
        });
        // build raw transaction
        let raw_txn = json!({
            "sender": sender.address(),
            "sequence_number": sequence_number.to_string(),
            "max_gas_amount": max_gas_amount.to_string(),
            "gas_unit_price": gas_unit_price.to_string(),
            "expiration_timestamp_secs": expiration_timestamp.to_string(),
            "payload": payload
        });
        Ok(raw_txn)
    }
    
    /// build token transfer
    pub async fn create_token_transfer_tx(
        client: Arc<AptosClient>,
        sender: Wallet,
        recipient: &str,
        token_type: &str,
        amount: u64,
        sequence_number: Option<u64>,
        expiration_secs: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
    ) -> Result<Value, String> {
        let chain_id = client.get_chain_info().await.unwrap().chain_id;
        let sequence_number = match sequence_number {
            Some(seq) => seq,
            None => {
                client
                    .get_account_sequence_number(&sender.address()?)
                    .await?
            }
        };
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expiration_timestamp = current_timestamp + expiration_secs;
        // build transaction payload
        let payload = json!({
            "type": "entry_function_payload",
            "function": "0x1::coin::transfer",
            "type_arguments": [token_type],
            "arguments": [recipient, amount.to_string()]
        });
        // build raw transaction
        let raw_txn = json!({
            "sender": sender.address().unwrap(),
            "sequence_number": sequence_number.to_string(),
            "max_gas_amount": max_gas_amount.to_string(),
            "gas_unit_price": gas_unit_price.to_string(),
            "expiration_timestamp_secs": expiration_timestamp.to_string(),
            "payload": payload,
            "chain_id": chain_id
        });
        Ok(raw_txn)
    }

    /// create sign and submit transfer tx
    pub async fn create_sign_submit_transfer_tx(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        recipient: &str,
        amount: u64,
        sequence_number: Option<u64>,
        expiration_secs: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
    ) -> Result<String, String> {
        // build raw transaction
        let raw_txn = Trade::create_transfer_tx(
            Arc::clone(&client),
            Arc::clone(&wallet),
            recipient,
            amount,
            sequence_number,
            expiration_secs,
            max_gas_amount,
            gas_unit_price,
        )
        .await
        .unwrap();
        // serialize transaction and sign
        let message_to_sign = serialize_transaction_and_sign(&raw_txn)?;
        // wallet sign
        match wallet.sign(&message_to_sign) {
            Ok(signature_bytes) => {
                // create signed transaction tx
                match Trade::create_signed_transaction_tx(
                    Arc::clone(&wallet),
                    raw_txn,
                    signature_bytes,
                ) {
                    Ok(signed_txn) => {
                        // submit transaction
                        match client.submit_transaction(&signed_txn).await {
                            Ok(result) => {
                                return Ok(result.hash);
                            }
                            Err(e) => return Err(format!("submit transaction error: {:?}", e)),
                        }
                    }
                    Err(e) => return Err(format!("build signed transaction error: {:?}", e)),
                }
            }
            Err(e) => {
                return Err(format!("wallet sign error:{:?}", e).to_string());
            }
        }
    }

    /// build call contract tx
    pub async fn create_call_contract_tx(
        client: Arc<AptosClient>,
        sender: Arc<Wallet>,
        sequence_number: Option<u64>,
        expiration_secs: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
        payload: EntryFunctionPayload,
    ) -> Result<Value, String> {
        let sequence_number = match sequence_number {
            Some(seq) => seq,
            None => client
                .get_account_sequence_number(&sender.address().unwrap())
                .await
                .unwrap(),
        };
        let chain_id = client.get_chain_info().await.unwrap().chain_id;
        // current timestamp
        let current_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // expiration time
        let expiration_timestamp = current_timestamp + expiration_secs;
        // build raw transaction
        let raw_txn = json!({
            "sender": sender.address()?,
            "sequence_number": sequence_number.to_string(),
            "max_gas_amount": max_gas_amount.to_string(),
            "gas_unit_price": gas_unit_price.to_string(),
            "expiration_timestamp_secs": expiration_timestamp.to_string(),
            "payload": payload,
            "chain_id": chain_id
        });
        Ok(raw_txn)
    }

    /// create customize call contract tx
    pub async fn create_customize_call_contract_tx(
        client: Arc<AptosClient>,
        module_address: &str,
        module_name: &str,
        function_name: &str,
        type_arguments: Vec<String>,
        arguments: Vec<Value>,
        sender: Arc<Wallet>,
        sequence_number: Option<u64>,
        expiration_secs: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
    ) -> Result<Value, String> {
        let function_str = format!("{}::{}::{}", module_address, module_name, function_name);
        let function_vec = function_str.as_bytes().to_vec();
        let mut type_args: Vec<Vec<u8>> = Vec::new();
        type_arguments
            .iter()
            .for_each(|s| type_args.push(s.as_bytes().to_vec()));
        let mut args: Vec<Vec<u8>> = Vec::new();
        arguments
            .iter()
            .for_each(|s| args.push(s.as_str().unwrap().to_string().as_bytes().to_vec()));
        let payload = EntryFunctionPayload {
            module_address: address_to_bytes(module_address).unwrap().to_vec(),
            module_name: address_to_bytes(module_name).unwrap().to_vec(),
            function_name: function_vec,
            type_arguments: type_args,
            arguments: args,
        };
        Trade::create_call_contract_tx(
            client,
            sender,
            sequence_number,
            expiration_secs,
            max_gas_amount,
            gas_unit_price,
            payload,
        )
        .await
    }

    ///  build signed transaction
    pub fn create_signed_transaction_tx(
        wallet: Arc<Wallet>,
        raw_txn: Value,
        signature: Vec<u8>,
    ) -> Result<Value, String> {
        let public_key_hex = wallet
            .public_key_hex()
            .map_err(|e| format!("get public key hex: {}", e))?;
        Ok(json!({
            "transaction": raw_txn,
            "signature": {
                "type": "ed25519_signature",
                "public_key": public_key_hex,
                "signature": hex::encode(signature)
            }
        }))
    }
}
