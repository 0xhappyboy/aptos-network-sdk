use crate::{
    Aptos,
    types::{ContractCall, EntryFunctionPayload},
    wallet::Wallet,
};
use aptos_network_tool::{address::address_to_bytes, signature::serialize_transaction_and_sign};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::Semaphore;

pub struct Trade;

impl Trade {
    /// build transfer info
    pub async fn create_transfer_tx(
        client: Arc<Aptos>,
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
        client: Arc<Aptos>,
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
        client: Arc<Aptos>,
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
        client: Arc<Aptos>,
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
        client: Arc<Aptos>,
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

    /// Retrieves transaction history for a specified address with pagination support
    ///
    /// # Params
    /// client - Aptos Client
    /// address - address
    /// query - query:
    ///     - start - page
    ///     - limit - page size
    ///
    /// # Returns
    /// Ok(Vec<Transaction>) - Transaction vec
    /// Err(String) - Error
    ///
    /// # Examples
    /// ```rust
    /// use std::sync::Arc;
    ///
    /// let client = Arc::new(Aptos::new(AptosType::Mainnet));
    /// let query = TransactionQuery {
    ///     start: Some(0),
    ///     limit: Some(10)
    /// };
    ///
    /// match Trade::get_address_transactions(client, "0x1234...", query).await {
    ///     Ok(transactions) => println!("Search {} transactions", transactions.len()),
    ///     Err(e) => println!("Error: {}", e),
    /// }
    /// ```
    pub async fn get_address_transactions(
        client: Arc<Aptos>,
        address: &str,
        query: TransactionQuery,
    ) -> Result<Vec<TransactionInfo>, String> {
        client
            .get_account_transaction_vec(address, query.limit, query.start)
            .await
    }

    /// Filters transactions from address_a to only include those involving address_b
    ///
    /// # Params
    /// client - aptos client
    /// address_a - The primary account address to fetch transactions from
    /// address_b - Address B used as filtering condition
    /// limit - data limit
    /// start - starting sequence number for pagination (Optional)
    ///
    /// # Returns
    /// Ok(Vec<Transaction>) - Filtered vector of transactions where both addresses are involved
    /// Err(String) - Error message if the request fails
    ///
    /// # Examples
    /// ```
    /// use std::sync::Arc;
    ///
    /// let client = Arc::new(Aptos::new(AptosType::Mainnet));
    ///
    /// // Find all transactions where 0x1234... and 0x5678... interacted
    /// match Trade::get_transactions_involving_both_addresses(
    ///     client,
    ///     "0x1234...",
    ///     "0x5678...",
    ///     Some(50),  // Limit to 50 transactions
    ///     None       // Start from most recent
    /// ).await {
    ///     Ok(shared_transactions) => {
    ///         println!("Search {} shared transactions", shared_transactions.len());
    ///     },
    ///     Err(e) => println!("Error: {}", e),
    /// }
    /// ```
    ///
    pub async fn get_transactions_involving_both_addresses(
        client: Arc<Aptos>,
        address_a: &str,
        address_b: &str,
        limit: Option<u64>,
        start: Option<u64>,
    ) -> Result<Vec<TransactionInfo>, String> {
        let query = TransactionQuery { start, limit };
        let transactions = Self::get_address_transactions(client, address_a, query).await?;
        let filtered_transactions: Vec<TransactionInfo> = transactions
            .into_iter()
            .filter(|txn| Self::transaction_involves_address(txn, address_b))
            .collect();
        Ok(filtered_transactions)
    }

    /// Retrieves transactions where address_b is the sender and address_a is the recipient
    ///
    /// This function specifically finds transfer transactions where `address_b` (payer) sent funds
    /// to `address_a` (recipient). It searches through `address_b`'s transaction history and
    /// filters for coin transfer operations targeting `address_a`.
    ///
    /// # Params
    /// client - aptos client
    /// address_a - The primary account address to fetch transactions from
    /// address_b - Address B used as filtering condition
    /// limit - data limit
    /// start - starting sequence number for pagination (Optional)
    ///
    /// # Returns
    /// Ok(Vec<Transaction>) - Transaction vec
    /// Err(String) - Error message
    ///
    /// # Examples
    /// ```
    /// use std::sync::Arc;
    ///
    /// let client = Arc::new(Aptos::new(AptosType::Mainnet));
    ///
    /// // Find all payments from Alice (0x5678...) to Bob (0x1234...)
    /// match Trade::get_transactions_by_recipient_sender(
    ///     client,
    ///     "0x1234...",  // address A - recipient
    ///     "0x5678...",  // address B - sender
    ///     Some(100),    // Limit to 100 transactions
    ///     None          // Start from most recent
    /// ).await {
    ///     Ok(payments) => {
    ///         println!("Found {} payments from Alice to Bob", payments.len());
    ///         for payment in payments {
    ///             if let Some(transfer_info) = Trade::get_transfer_info(&payment) {
    ///                 println!("Amount: {} {}", transfer_info.amount, transfer_info.token_type);
    ///             }
    ///         }
    ///     },
    ///     Err(e) => println!("Error: {}", e),
    /// }
    /// ```
    ///
    pub async fn get_transactions_by_recipient_sender(
        client: Arc<Aptos>,
        address_a: &str, // Receiver
        address_b: &str, // Payer
        limit: Option<u64>,
        start: Option<u64>,
    ) -> Result<Vec<TransactionInfo>, String> {
        let query = TransactionQuery { start, limit };
        let transactions =
            Self::get_address_transactions(Arc::clone(&client), address_b, query).await?;
        let filtered_transactions: Vec<TransactionInfo> = transactions
            .into_iter()
            .filter(|txn| Self::is_transfer_from_to(txn, address_b, address_a))
            .collect();
        Ok(filtered_transactions)
    }

    /// Check if the transaction involves the specified address
    fn transaction_involves_address(transaction: &TransactionInfo, address: &str) -> bool {
        match &transaction.transaction_type {
            TransactionType::UserTransaction(user_txn) => {
                // Check sender
                if user_txn.sender == address {
                    return true;
                }
                // Check the address parameter in the payload
                Self::payload_contains_address(&user_txn.payload, address)
            }
            TransactionType::PendingTransaction(pending_txn) => {
                pending_txn.sender == address
                    || Self::payload_contains_address(&pending_txn.payload, address)
            }
            _ => false,
        }
    }

    /// Check if the transaction is a transfer from from_address to to_address
    fn is_transfer_from_to(
        transaction: &TransactionInfo,
        from_address: &str,
        to_address: &str,
    ) -> bool {
        match &transaction.transaction_type {
            TransactionType::UserTransaction(user_txn) => {
                if user_txn.sender != from_address {
                    return false;
                }
                Self::is_transfer_to_address(&user_txn.payload, to_address)
            }
            TransactionType::PendingTransaction(pending_txn) => {
                if pending_txn.sender != from_address {
                    return false;
                }
                Self::is_transfer_to_address(&pending_txn.payload, to_address)
            }
            _ => false,
        }
    }

    /// Check if the payload contains the specified address
    fn payload_contains_address(payload: &Payload, address: &str) -> bool {
        for arg in &payload.arguments {
            if let Some(arg_str) = arg.as_str() {
                if arg_str == address {
                    return true;
                }
            }
        }
        false
    }

    /// Check if the payload is a transfer to the specified address
    fn is_transfer_to_address(payload: &Payload, recipient_address: &str) -> bool {
        if payload.function.ends_with("::coin::transfer") {
            if let Some(first_arg) = payload.arguments.first() {
                if let Some(recipient) = first_arg.as_str() {
                    return recipient == recipient_address;
                }
            }
        }
        false
    }

    /// Get user transaction details
    pub fn get_user_transaction(transaction: &TransactionInfo) -> Option<&UserTransaction> {
        match &transaction.transaction_type {
            TransactionType::UserTransaction(user_txn) => Some(user_txn),
            _ => None,
        }
    }

    /// Get transfer information in the transaction
    pub fn get_transfer_info(transaction: &TransactionInfo) -> Option<TransferInfo> {
        let user_txn = Self::get_user_transaction(transaction)?;
        if user_txn.payload.function.ends_with("::coin::transfer") {
            if user_txn.payload.arguments.len() >= 2 {
                let recipient = user_txn.payload.arguments[0].as_str()?.to_string();
                let amount = user_txn.payload.arguments[1].as_str()?.parse().ok()?;
                // Extract token type
                let token_type = if !user_txn.payload.type_arguments.is_empty() {
                    user_txn.payload.type_arguments[0].clone()
                } else {
                    "0x1::aptos_coin::AptosCoin".to_string()
                };
                Some(TransferInfo {
                    from: user_txn.sender.clone(),
                    to: recipient,
                    amount,
                    token_type,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get events of a specific type in transaction events
    pub fn get_events_by_type<'a>(
        transaction: &'a TransactionInfo,
        event_type: &str,
    ) -> Vec<&'a Event> {
        transaction
            .events
            .iter()
            .filter(|event| event.r#type.contains(event_type))
            .collect()
    }

    /// Analyze resource changes in transactions
    pub fn analyze_resource_changes(transaction: &TransactionInfo) -> ResourceChanges {
        let mut changes = ResourceChanges::default();
        for change in &transaction.changes {
            match change.change_type.as_str() {
                "write_resource" => {
                    if let Some(_data) = &change.data {
                        changes.resources_modified += 1;
                    }
                }
                "write_table_item" => {
                    changes.table_items_modified += 1;
                }
                "delete_resource" => {
                    changes.resources_deleted += 1;
                }
                "delete_table_item" => {
                    changes.table_items_deleted += 1;
                }
                _ => {}
            }
        }
        changes
    }
}

/// batch transaction processor
pub struct BatchTradeHandle;

impl BatchTradeHandle {
    /// Processing batch transactions with concurrency control
    pub async fn process_batch(
        client: Arc<Aptos>,
        wallet: Arc<Wallet>,
        calls: Vec<ContractCall>,
        concurrency: usize,
    ) -> Result<Vec<Value>, String> {
        let semaphore = Arc::new(Semaphore::new(concurrency));
        let mut tasks = Vec::new();
        for call in calls {
            let client_clone = Arc::clone(&client);
            let wallet_clone = Arc::clone(&wallet);
            let semaphore_clone = Arc::clone(&semaphore);

            let task = async move {
                let _permit = semaphore_clone.acquire().await.map_err(|e| e.to_string())?;
                match crate::contract::Contract::write(client_clone, wallet_clone, call).await {
                    Ok(result) => Ok(json!(result)),
                    Err(e) => Err(e),
                }
            };
            tasks.push(task);
        }
        let results = join_all(tasks).await;
        let mut final_results = Vec::new();
        for result in results {
            match result {
                Ok(value) => final_results.push(value),
                Err(e) => final_results.push(json!({
                    "success": false,
                    "error": e
                })),
            }
        }
        Ok(final_results)
    }

    /// Read resources in batches
    pub async fn batch_get_resources(
        client: Arc<Aptos>,
        addresses: Vec<String>,
        resource_types: Vec<&str>,
    ) -> Result<HashMap<String, HashMap<String, Option<Value>>>, String> {
        let mut all_results = HashMap::new();
        for address in addresses {
            match crate::contract::Contract::batch_get_resources(
                Arc::clone(&client),
                &address,
                resource_types.clone(),
            )
            .await
            {
                Ok(resources) => {
                    all_results.insert(address, resources);
                }
                Err(e) => {
                    eprintln!("Failed to get resources for address: {}", e);
                }
            }
        }
        Ok(all_results)
    }
}

/// Represents a transaction on the Aptos blockchain
/// Contains all relevant information about a transaction including metadata, payload, and execution results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub version: String,
    pub hash: String,
    #[serde(default)]
    pub state_change_hash: String,
    #[serde(default)]
    pub event_root_hash: String,
    pub state_checkpoint_hash: Option<String>,
    #[serde(default)]
    pub gas_used: String,
    pub success: bool,
    #[serde(default)]
    pub vm_status: String,
    #[serde(default)]
    pub accumulator_root_hash: String,
    #[serde(default)]
    pub changes: Vec<WriteSetChange>,
    #[serde(default)]
    pub events: Vec<Event>,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub max_gas_amount: Option<String>,
    #[serde(flatten)]
    pub transaction_type: TransactionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TransactionType {
    #[serde(rename = "pending_transaction")]
    PendingTransaction(PendingTransaction),
    #[serde(rename = "user_transaction")]
    UserTransaction(UserTransaction),
    #[serde(rename = "genesis_transaction")]
    GenesisTransaction(GenesisTransaction),
    #[serde(rename = "block_metadata_transaction")]
    BlockMetadataTransaction(BlockMetadataTransaction),
    #[serde(rename = "state_checkpoint_transaction")]
    StateCheckpointTransaction(StateCheckpointTransaction),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTransaction {
    pub hash: String,
    pub sender: String,
    pub sequence_number: String,
    pub max_gas_amount: String,
    pub gas_unit_price: String,
    pub expiration_timestamp_secs: String,
    pub payload: Payload,
    pub signature: Option<Signature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTransaction {
    pub sender: String,
    pub sequence_number: String,
    #[serde(default)]
    pub max_gas_amount: Option<String>,
    #[serde(default)]
    pub gas_unit_price: Option<String>,
    #[serde(default)]
    pub expiration_timestamp_secs: Option<String>,
    pub payload: Payload,
    pub signature: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisTransaction {
    pub payload: Payload,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMetadataTransaction {
    pub id: String,
    pub epoch: String,
    pub round: String,
    pub proposer: String,
    pub failed_proposer_indices: Vec<u64>,
    pub previous_block_votes_bitvec: Vec<u8>,
    pub timestamp: String,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateCheckpointTransaction {
    pub timestamp: String,
    pub version: String,
    pub hash: String,
    pub state_change_hash: String,
    pub event_root_hash: String,
    pub state_checkpoint_hash: Option<String>,
    pub gas_used: String,
    pub success: bool,
    pub vm_status: String,
    pub accumulator_root_hash: String,
    pub changes: Vec<WriteSetChange>,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    #[serde(rename = "type")]
    pub payload_type: String,
    pub function: String,
    pub type_arguments: Vec<String>,
    pub arguments: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<Code>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Code {
    pub bytecode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Signature {
    #[serde(rename = "ed25519_signature")]
    Ed25519 {
        public_key: String,
        signature: String,
    },
    #[serde(rename = "multi_ed25519_signature")]
    MultiEd25519 {
        public_keys: Vec<String>,
        signatures: Vec<String>,
        threshold: u8,
    },
    #[serde(rename = "single_key_signature")]
    SingleKey {
        public_key: String,
        signature: String,
    },
    #[serde(rename = "multi_key_signature")]
    MultiKey {
        public_keys: Vec<String>,
        signatures: Vec<String>,
        threshold: u8,
    },
    #[serde(rename = "fee_payer_signature")]
    FeePayer {
        sender: Box<Signature>,
        #[serde(default)]
        fee_payer: Option<Box<Signature>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub guid: Guid,
    pub sequence_number: String,
    pub r#type: String,
    pub data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guid {
    pub creation_number: String,
    pub account_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct WriteSetChange {
    #[serde(rename = "type")]
    pub change_type: String,
    pub address: Option<String>,
    pub state_key_hash: String,
    pub data: Option<Value>,
    pub handle: Option<String>,
    pub key: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTransactionRequest {
    pub sender: String,
    pub sequence_number: String,
    pub max_gas_amount: String,
    pub gas_unit_price: String,
    pub expiration_timestamp_secs: String,
    pub payload: Payload,
    pub signature: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionQuery {
    pub start: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferInfo {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceChanges {
    pub resources_modified: usize,
    pub resources_deleted: usize,
    pub table_items_modified: usize,
    pub table_items_deleted: usize,
}

impl TransactionInfo {
    /// Check if the transaction was successful
    pub fn is_successful(&self) -> bool {
        self.success
    }

    /// Get transaction timestamp
    pub fn get_timestamp(&self) -> Option<u64> {
        self.timestamp.clone().unwrap_or(0.to_string()).parse().ok()
    }

    /// Get the amount of gas used
    pub fn get_gas_used(&self) -> Option<u64> {
        self.gas_used.parse().ok()
    }

    /// Check whether it is a user transaction
    pub fn is_user_transaction(&self) -> bool {
        matches!(self.transaction_type, TransactionType::UserTransaction(_))
    }

    /// Get the sender address in this transaction.
    pub fn get_sender(&self) -> Option<&str> {
        match &self.transaction_type {
            TransactionType::UserTransaction(user_txn) => Some(&user_txn.sender),
            TransactionType::PendingTransaction(pending_txn) => Some(&pending_txn.sender),
            _ => None,
        }
    }

    fn extract_received_from_event(event: &Event) -> Vec<(String, u64)> {
        let mut result = Vec::new();
        if let serde_json::Value::Object(data) = &event.data {
            if event.r#type.contains("Swap") {
                if let (Some(amount_value), Some(token_value)) =
                    (data.get("amount_out"), data.get("to_token"))
                {
                    if let (Some(amount), Some(token_str)) = (
                        Self::parse_amount_simple(amount_value),
                        token_value.as_str(),
                    ) {
                        result.push((token_str.to_string(), amount));
                    }
                }
                let output_combos = [
                    ("amount_y_out", "token_y"),
                    ("output_amount", "output_token"),
                    ("amount1_out", "token1"),
                ];
                for (amount_field, token_field) in &output_combos {
                    if let (Some(amount_value), Some(token_value)) =
                        (data.get(*amount_field), data.get(*token_field))
                    {
                        if let (Some(amount), Some(token_str)) = (
                            Self::parse_amount_simple(amount_value),
                            token_value.as_str(),
                        ) {
                            result.push((token_str.to_string(), amount));
                        }
                    }
                }
            }
            if event.r#type.contains("fungible_asset::Deposit") {
                if let Some(amount_value) = data.get("amount") {
                    if let Some(amount) = Self::parse_amount_simple(amount_value) {
                        let token_type = Self::infer_token_from_event_type(event);
                        if let Some(token) = token_type {
                            result.push((token.clone(), amount));
                        }
                    }
                }
            }
        }
        result
    }

    fn extract_spent_from_event(event: &Event) -> Vec<(String, u64)> {
        let mut result = Vec::new();
        if let serde_json::Value::Object(data) = &event.data {
            if event.r#type.contains("Swap") {
                if let (Some(amount_value), Some(token_value)) =
                    (data.get("amount_in"), data.get("from_token"))
                {
                    if let (Some(amount), Some(token_str)) = (
                        Self::parse_amount_simple(amount_value),
                        token_value.as_str(),
                    ) {
                        result.push((token_str.to_string(), amount));
                    }
                }
                let input_combos = [
                    ("amount_x_in", "token_x"),
                    ("amount0_in", "token0"),
                    ("input_amount", "input_token"),
                ];
                for (amount_field, token_field) in &input_combos {
                    if let (Some(amount_value), Some(token_value)) =
                        (data.get(*amount_field), data.get(*token_field))
                    {
                        if let (Some(amount), Some(token_str)) = (
                            Self::parse_amount_simple(amount_value),
                            token_value.as_str(),
                        ) {
                            result.push((token_str.to_string(), amount));
                        }
                    }
                }
            }
            if event.r#type.contains("fungible_asset::Withdraw") {
                if let Some(amount_value) = data.get("amount") {
                    if let Some(amount) = Self::parse_amount_simple(amount_value) {
                        let token_type = Self::infer_token_from_event_type(event);
                        if let Some(token) = token_type {
                            result.push((token.clone(), amount));
                        }
                    }
                }
            }
        }
        result
    }

    fn infer_token_from_event_type(event: &Event) -> Option<String> {
        let event_type = &event.r#type;
        if event_type.contains("aptos_coin") {
            Some("0x1::aptos_coin::AptosCoin".to_string())
        } else if event_type.contains("usdt") || event_type.contains("USDt") {
            Some("0x1::usdt::USDT".to_string())
        } else if event_type.contains("EchoCoin002") {
            Some("0xe4ccb6d39136469f376242c31b34d10515c8eaaa38092f804db8e08a8f53c5b2::assets_v1::EchoCoin002".to_string())
        } else if event_type
            .contains("0x2ebb2ccac5e027a87fa0e2e5f656a3a4238d6a48d93ec9b610d570fc0aa0df12")
        {
            Some("0x2ebb2ccac5e027a87fa0e2e5f656a3a4238d6a48d93ec9b610d570fc0aa0df12".to_string())
        } else if event_type
            .contains("0x357b0b74bc833e95a115ad22604854d6b0fca151cecd94111770e5d6ffc9dc2b")
        {
            Some("0x357b0b74bc833e95a115ad22604854d6b0fca151cecd94111770e5d6ffc9dc2b".to_string())
        } else {
            None
        }
    }

    pub fn get_spent_token(&self) -> Option<(String, u64)> {
        if !self.success {
            return None;
        }
        for event in self.events.iter().rev() {
            if event.r#type.contains("Swap") {
                if let serde_json::Value::Object(data) = &event.data {
                    let input_pairs = [
                        ("amount_in", "from_token"),
                        ("amount_x_in", "token_x"),
                        ("amount0_in", "token0"),
                        ("input_amount", "input_token"),
                        ("amount", "coin_type"),
                    ];
                    for (amount_field, token_field) in &input_pairs {
                        if let (Some(amount_value), Some(token_value)) =
                            (data.get(*amount_field), data.get(*token_field))
                        {
                            if let (Some(amount), Some(token_str)) = (
                                Self::parse_amount_simple(amount_value),
                                Self::extract_token_string(token_value),
                            ) {
                                if amount > 0 {
                                    return Some((token_str, amount));
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    pub fn get_received_token(&self) -> Option<(String, u64)> {
        if !self.success {
            return None;
        }
        for event in self.events.iter().rev() {
            if event.r#type.contains("Swap") {
                if let serde_json::Value::Object(data) = &event.data {
                    let output_pairs = [
                        ("amount_out", "to_token"),
                        ("amount_y_out", "token_y"),
                        ("amount1_out", "token1"),
                        ("output_amount", "output_token"),
                        ("amount", "coin_type"),
                    ];
                    for (amount_field, token_field) in &output_pairs {
                        if let (Some(amount_value), Some(token_value)) =
                            (data.get(*amount_field), data.get(*token_field))
                        {
                            if let (Some(amount), Some(token_str)) = (
                                Self::parse_amount_simple(amount_value),
                                Self::extract_token_string(token_value),
                            ) {
                                if amount > 0 {
                                    return Some((token_str, amount));
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn guess_decimals_from_amount(amount: u64) -> u8 {
        let amount_str = amount.to_string();
        let len = amount_str.len();
        if len > 6 && amount_str.ends_with("000000") {
            return 6;
        }
        if len > 8 && amount_str.ends_with("00000000") {
            return 8;
        }
        if amount > 1_000_000_000_000 {
            return 6;
        } else if amount > 10_000_000 && amount < 100_000_000_000 {
            return 8;
        } else {
            return 6;
        }
    }

    fn extract_token_string(value: &serde_json::Value) -> Option<String> {
        match value {
            serde_json::Value::String(s) => match s.as_str() {
                "0xa" => Some("0x1::aptos_coin::AptosCoin".to_string()),
                _ => Some(s.clone()),
            },
            serde_json::Value::Object(obj) => {
                for field in ["inner", "value", "address", "token"] {
                    if let Some(inner_value) = obj.get(field) {
                        if let Some(result) = Self::extract_token_string(inner_value) {
                            return Some(result);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn get_spent_token_eth(&self) -> Option<(String, f64)> {
        self.get_spent_token().map(|(token, amount)| {
            let decimals = Self::guess_decimals_from_amount(amount);
            let decimal_amount = amount as f64 / 10_u64.pow(decimals as u32) as f64;
            (token, decimal_amount)
        })
    }

    pub fn get_received_token_eth(&self) -> Option<(String, f64)> {
        self.get_received_token().map(|(token, amount)| {
            let decimals = Self::guess_decimals_from_amount(amount);
            let decimal_amount = amount as f64 / 10_u64.pow(decimals as u32) as f64;
            (token, decimal_amount)
        })
    }

    fn parse_amount_simple(value: &serde_json::Value) -> Option<u64> {
        if let Some(s) = value.as_str() {
            if let Ok(n) = s.parse::<u64>() {
                return Some(n);
            }
        }
        if let Some(n) = value.as_u64() {
            return Some(n);
        }
        if let Some(n) = value.as_i64() {
            if n >= 0 {
                return Some(n as u64);
            }
        }
        None
    }

    pub fn getDirection(&self) -> String {
        match (self.get_spent_token_eth(), self.get_received_token_eth()) {
            (Some((spent_token, _)), Some((received_token, _))) => {
                if spent_token.contains("EchoCoin002") && received_token.contains("aptos_coin") {
                    "BUY".to_string()
                } else if spent_token.contains("aptos_coin")
                    && received_token.contains("EchoCoin002")
                {
                    "SELL".to_string()
                } else {
                    "SWAP".to_string()
                }
            }
            _ => "TRANSFER".to_string(),
        }
    }

    fn get_decimals_for_token(token: &str) -> u8 {
        if token.contains("EchoCoin002")
            || token.contains("0x9da434d9b873b5159e8eeed70202ad22dc075867a7793234fbc981b63e119")
        {
            6
        } else if token.contains("aptos_coin") || token == "0xa" {
            8
        } else if token
            .contains("0x2ebb2ccac5e027a87fa0e2e5f656a3a4238d6a48d93ec9b610d570fc0aa0df12")
        {
            8
        } else if token
            .contains("0x357b0b74bc833e95a115ad22604854d6b0fca151cecd94111770e5d6ffc9dc2b")
        {
            6
        } else {
            8
        }
    }

    pub fn calculate_all_token_balances(&self) {
        let mut spent_map: HashMap<String, u64> = HashMap::new();
        let mut received_map: HashMap<String, u64> = HashMap::new();
        for event in &self.events {
            let spent = Self::extract_spent_from_event(event);
            for (token, amount) in spent {
                *spent_map.entry(token).or_insert(0) += amount;
            }
        }
        for event in &self.events {
            let received = Self::extract_received_from_event(event);
            for (token, amount) in received {
                *received_map.entry(token).or_insert(0) += amount;
            }
        }
        for (token, total) in &spent_map {
            let decimals = Self::get_decimals_for_token(token);
        }
        for (token, total) in &received_map {
            let decimals = Self::get_decimals_for_token(token);
        }
        let all_tokens: HashSet<_> = spent_map.keys().chain(received_map.keys()).collect();
        for token in all_tokens {
            let spent = spent_map.get(token).copied().unwrap_or(0);
            let received = received_map.get(token).copied().unwrap_or(0);
            let net = received as i128 - spent as i128;
            if net != 0 {
                let decimals = Self::get_decimals_for_token(token);
            }
        }
    }

    pub fn get_liquidity_pool_addresses(&self) -> Vec<String> {
        let mut pool_addresses = Vec::new();
        for event in &self.events {
            let event_type = &event.r#type;
            if event_type.contains("Pool") || event_type.contains("Swap") {
                if let guid = &event.guid {
                    pool_addresses.push(guid.account_address.clone());
                }
                if let serde_json::Value::Object(data) = &event.data {
                    let possible_fields = [
                        "pool_address",
                        "pool",
                        "pair",
                        "liquidity_pool",
                        "address",
                        "contract_address",
                        "dex_address",
                    ];
                    for field in &possible_fields {
                        if let Some(addr_value) = data.get(*field) {
                            if let Some(addr_str) = addr_value.as_str() {
                                pool_addresses.push(addr_str.to_string());
                                break;
                            }
                        }
                    }
                }
            }
        }
        pool_addresses.sort();
        pool_addresses.dedup();
        pool_addresses
    }

    pub fn get_dex_names(&self) -> Vec<String> {
        let mut dex_names = Vec::new();
        if let TransactionType::UserTransaction(user_txn) = &self.transaction_type {
            let function = &user_txn.payload.function;
            if function.contains("panora_swap") {
                dex_names.push("Panora Exchange".to_string());
            }
            if function.contains("pancake") {
                dex_names.push("PancakeSwap".to_string());
            }
            if function.contains("hyperion") {
                dex_names.push("Hyperion".to_string());
            }
            if function.contains("tapp") {
                dex_names.push("Tapp Exchange".to_string());
            }
            if function.contains("cellana") {
                dex_names.push("Cellana Finance".to_string());
            }
        }
        for event in &self.events {
            let event_type = &event.r#type;

            if event_type.contains("panora") && !dex_names.contains(&"Panora Exchange".to_string())
            {
                dex_names.push("Panora Exchange".to_string());
            }
            if event_type.contains("pancake") && !dex_names.contains(&"PancakeSwap".to_string()) {
                dex_names.push("PancakeSwap".to_string());
            }
            if event_type.contains("hyperion") && !dex_names.contains(&"Hyperion".to_string()) {
                dex_names.push("Hyperion".to_string());
            }
            if event_type.contains("tapp") && !dex_names.contains(&"Tapp Exchange".to_string()) {
                dex_names.push("Tapp Exchange".to_string());
            }
            if event_type.contains("cellana") && !dex_names.contains(&"Cellana Finance".to_string())
            {
                dex_names.push("Cellana Finance".to_string());
            }
            if let serde_json::Value::Object(data) = &event.data {
                let dex_fields = ["dex", "exchange", "platform", "protocol"];
                for field in &dex_fields {
                    if let Some(dex_value) = data.get(*field) {
                        if let Some(dex_str) = dex_value.as_str() {
                            let dex_name = dex_str.to_string();
                            if !dex_names.contains(&dex_name) {
                                dex_names.push(dex_name);
                            }
                        }
                    }
                }
            }
        }
        if dex_names.is_empty() {
            let pools = self.get_liquidity_pool_addresses();
            for pool in pools {
                if pool.contains("0x1c3206") {
                    dex_names.push("Panora Exchange".to_string());
                } else if pool.contains("0x2788f4") {
                    dex_names.push("Hyperion".to_string());
                } else if pool.contains("0x85d333") {
                    dex_names.push("Tapp Exchange".to_string());
                } else if pool.contains("0xd18e39") {
                    dex_names.push("Cellana Finance".to_string());
                }
            }
        }
        dex_names.sort();
        dex_names.dedup();
        dex_names
    }
}

#[cfg(test)]
mod tests {
    use crate::AptosType;

    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_specific_transaction() {
        let client = Aptos::new(AptosType::Mainnet);
        let known_tx_hash = "0x280a3e0c7e2ab02de2f8052441464fd8b351804c9d336ec988d75b59446ecfdc";
        let result = client.get_transaction_info_by_hash(known_tx_hash).await;
        match result {
            Ok(tx) => {
                println!("Spent {:?}", tx.get_spent_token_eth());
                println!("Received {:?}", tx.get_received_token_eth());
                println!("Liquidity Pool {:?}", tx.get_liquidity_pool_addresses());
            }
            Err(e) => {
                println!("❌ error: {}", e);
            }
        }
    }
}
