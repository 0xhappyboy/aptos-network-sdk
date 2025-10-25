use aptos_network_tool::address::address_to_bytes;
use futures::future::join_all;
// src/contract.rs
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::{
    AptosClient,
    trade::Trade,
    types::{
        ContractCall, ContractReadResult, ContractWriteResult, EntryFunctionPayload, Event,
        ViewRequest,
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
                        if let Ok(current_seq) = event.sequence_number.parse::<u64>() {
                            if let Some(last_seq) = last_sequence_number {
                                if current_seq > last_seq {
                                    callback(Ok(event.data.clone()));
                                }
                            } else {
                                callback(Ok(event.data.clone()));
                            }
                            // update last sequence number
                            last_sequence_number = Some(current_seq);
                        }
                    }
                }
                Err(e) => callback(Err(format!("no event exists: {:?}", e).to_string())),
            }
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;
        }
    }

    /// Event Listener - contains complete event information
    pub async fn listen_events_all_info(
        client: Arc<AptosClient>,
        address: &str,
        event_type: &str,
        callback: impl Fn(Result<Event, String>),
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
                        if let Ok(current_seq) = event.sequence_number.parse::<u64>() {
                            if let Some(last_seq) = last_sequence_number {
                                if current_seq > last_seq {
                                    callback(Ok(event.clone()));
                                }
                            } else {
                                callback(Ok(event.clone()));
                            }
                            last_sequence_number = Some(current_seq);
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

    /// Get contract status snapshot
    pub async fn get_contract_state_snapshot(
        client: Arc<AptosClient>,
        address: &str,
        resource_types: Vec<&str>,
    ) -> Result<HashMap<String, Option<Value>>, String> {
        let mut snapshot = HashMap::new();
        for resource_type in resource_types {
            match Self::get_contract_resource(Arc::clone(&client), address, resource_type).await {
                Ok(Some(data)) => {
                    snapshot.insert(resource_type.to_string(), Some(data));
                }
                Ok(None) => {
                    snapshot.insert(resource_type.to_string(), None);
                }
                Err(e) => {
                    snapshot.insert(resource_type.to_string(), None);
                    eprintln!("Error fetching resource {}: {}", resource_type, e);
                }
            }
        }
        Ok(snapshot)
    }

    /// Verify contract call parameters
    pub fn validate_contract_call(contract_call: &ContractCall) -> Result<(), String> {
        if contract_call.module_address.is_empty() {
            return Err("Module address cannot be empty".to_string());
        }
        if contract_call.module_name.is_empty() {
            return Err("Module name cannot be empty".to_string());
        }
        if contract_call.function_name.is_empty() {
            return Err("Function name cannot be empty".to_string());
        }
        // Verify address format
        if !contract_call.module_address.starts_with("0x") {
            return Err("Module address must start with 0x".to_string());
        }
        Ok(())
    }

    /// Estimating contract call gas fees
    pub async fn estimate_gas_cost(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        contract_call: &ContractCall,
    ) -> Result<u64, String> {
        // estimate gas
        let simulation_result = Self::simulate_call_contract(client, wallet, contract_call).await?;
        simulation_result
            .get("gas_used")
            .and_then(|g| g.as_str())
            .and_then(|g| g.parse().ok())
            .ok_or_else(|| "Failed to estimate gas cost".to_string())
    }

    /// Retry failed contract calls
    pub async fn retry_failed_call(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        contract_call: ContractCall,
        max_retries: u32,
        retry_delay_secs: u64,
    ) -> Result<ContractWriteResult, String> {
        let mut retries = 0;
        while retries < max_retries {
            match Self::write(
                Arc::clone(&client),
                Arc::clone(&wallet),
                contract_call.clone(),
            )
            .await
            {
                Ok(result) if result.success => return Ok(result),
                Ok(result) => {
                    eprintln!("Call failed on attempt {}: {:?}", retries + 1, result.error);
                }
                Err(e) => {
                    eprintln!("Error on attempt {}: {}", retries + 1, e);
                }
            }
            retries += 1;
            if retries < max_retries {
                tokio::time::sleep(Duration::from_secs(retry_delay_secs)).await;
            }
        }
        Err(format!("Failed after {} retries", max_retries))
    }

    /// Batch resource query
    pub async fn batch_get_resources(
        client: Arc<AptosClient>,
        address: &str,
        resource_types: Vec<&str>,
    ) -> Result<HashMap<String, Option<Value>>, String> {
        let mut tasks = Vec::new();
        for resource_type in resource_types {
            let client_clone = Arc::clone(&client);
            let address = address.to_string();
            let resource_type = resource_type.to_string();
            tasks.push(async move {
                match client_clone
                    .get_account_resource(&address, &resource_type)
                    .await
                {
                    Ok(Some(resource)) => (resource_type, Some(resource.data)),
                    Ok(None) => (resource_type, None),
                    Err(_) => (resource_type, None),
                }
            });
        }
        let results = join_all(tasks).await;
        let mut resource_map = HashMap::new();
        for (resource_type, data) in results {
            resource_map.insert(resource_type, data);
        }
        Ok(resource_map)
    }

    /// Batch call contract write function
    pub async fn batch_write(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        calls: Vec<ContractCall>,
    ) -> Result<Vec<Value>, String> {
        let mut results = Vec::new();
        for call in calls {
            match Self::write(Arc::clone(&client), Arc::clone(&wallet), call).await {
                Ok(result) => results.push(json!(result)),
                Err(e) => results.push(json!({
                    "success": false,
                    "error": e
                })),
            }
        }
        Ok(results)
    }

    /// Simulate contract call execution (estimate Gas)
    pub async fn simulate_call_contract(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        contract_call: &ContractCall,
    ) -> Result<Value, String> {
        let function = format!(
            "{}::{}::{}",
            contract_call.module_address, contract_call.module_name, contract_call.function_name
        );
        todo!();
        let payload = json!({
            "function": function,
            "type_arguments": contract_call.type_arguments,
            "arguments": contract_call.arguments,
            "sender": wallet.address().map_err(|e| e.to_string())?,
        });

        // test data
        todo!();
        Ok(json!({
            "gas_used": "1000",
            "success": true,
            "vm_status": "Executed successfully"
        }))
    }

    /// Get the ABI information of the contract
    pub async fn get_contract_abi(
        client: Arc<AptosClient>,
        module_address: &str,
        module_name: &str,
    ) -> Result<Option<Value>, String> {
        Ok(None)
    }

    /// Check if the contract has been published
    pub async fn is_contract_deployed(
        client: Arc<AptosClient>,
        module_address: &str,
        module_name: &str,
    ) -> Result<bool, String> {
        match Self::get_contract_abi(client, module_address, module_name).await {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(_) => Ok(false),
        }
    }

    /// Get contract event list
    pub async fn get_contract_events(
        client: Arc<AptosClient>,
        address: &str,
        event_handle: &str,
        limit: Option<u64>,
        start: Option<u64>,
    ) -> Result<Vec<Value>, String> {
        let events = client
            .get_account_event_vec(address, event_handle, limit, start)
            .await?;
        let value_events: Vec<Value> = events
            .into_iter()
            .map(|event| {
                json!({
                    "type": event.r#type,
                    "data": event.data,
                    "sequence_number": event.sequence_number,
                    "guid": event.guid
                })
            })
            .collect();
        Ok(value_events)
    }

    /// Parsing complex type parameters
    pub fn parse_complex_type_arguments(type_args: Vec<&str>) -> Vec<String> {
        type_args.into_iter().map(|s| s.to_string()).collect()
    }

    /// Constructing complex call parameters
    pub fn build_complex_arguments(args: Vec<&str>) -> Vec<Value> {
        args.into_iter()
            .map(|s| Value::String(s.to_string()))
            .collect()
    }

    /// Contract call result analyzer
    pub fn analyze_contract_result(result: &Value) -> HashMap<String, String> {
        let mut analysis = HashMap::new();
        if let Some(success) = result.get("success").and_then(|s| s.as_bool()) {
            analysis.insert(
                "status".to_string(),
                if success {
                    "success".to_string()
                } else {
                    "failed".to_string()
                },
            );
        }
        if let Some(gas_used) = result.get("gas_used").and_then(|g| g.as_str()) {
            analysis.insert("gas_used".to_string(), gas_used.to_string());
        }
        if let Some(error) = result.get("error").and_then(|e| e.as_str()) {
            analysis.insert("error".to_string(), error.to_string());
        }
        analysis
    }

    /// Release new contract module
    pub async fn deploy_contract(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        module_bytes: Vec<u8>,
        metadata: Option<Value>,
    ) -> Result<Value, String> {
        // Use existing transaction build and commit logic
        let contract_call = ContractCall {
            module_address: wallet.address().map_err(|e| e.to_string())?,
            module_name: "".to_string(), // Deploying a contract does not require a module name
            function_name: "deploy".to_string(),
            type_arguments: vec![],
            arguments: vec![json!(hex::encode(module_bytes))],
        };
        Self::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// Update a deployed contract
    pub async fn upgrade_contract(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        module_name: &str,
        new_module_bytes: Vec<u8>,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: wallet.address().map_err(|e| e.to_string())?,
            module_name: module_name.to_string(),
            function_name: "upgrade".to_string(),
            type_arguments: vec![],
            arguments: vec![json!(hex::encode(new_module_bytes))],
        };
        Self::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }
}

/// Contract tool
pub struct ContractUtils;

impl ContractUtils {
    /// Generate standard contract calls
    pub fn create_standard_call(
        module_address: &str,
        module_name: &str,
        function_name: &str,
        type_arguments: Vec<String>,
        arguments: Vec<Value>,
    ) -> ContractCall {
        ContractCall {
            module_address: module_address.to_string(),
            module_name: module_name.to_string(),
            function_name: function_name.to_string(),
            type_arguments,
            arguments,
        }
    }

    /// Parsing event data into a structured format
    pub fn parse_event_data(event_data: Value, event_schema: &[&str]) -> HashMap<String, Value> {
        let mut parsed = HashMap::new();

        if let Value::Object(map) = event_data {
            for (key, value) in map {
                if event_schema.contains(&key.as_str()) {
                    parsed.insert(key, value);
                }
            }
        }
        parsed
    }

    /// Calculate the contract call signature
    pub fn calculate_call_signature(contract_call: &ContractCall, nonce: &str) -> String {
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        hasher.update(contract_call.module_address.as_bytes());
        hasher.update(contract_call.module_name.as_bytes());
        hasher.update(contract_call.function_name.as_bytes());
        hasher.update(nonce.as_bytes());
        format!("0x{}", hex::encode(hasher.finalize()))
    }
}
