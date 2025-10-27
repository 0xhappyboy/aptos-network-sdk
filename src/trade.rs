use crate::{
    AptosClient,
    types::{ContractCall, EntryFunctionPayload},
    wallet::Wallet,
};
use aptos_network_tool::{address::address_to_bytes, signature::serialize_transaction_and_sign};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::Semaphore;

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
    /// let client = Arc::new(AptosClient::new(AptosClientType::Mainnet));
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
        client: Arc<AptosClient>,
        address: &str,
        query: TransactionQuery,
    ) -> Result<Vec<Transaction>, String> {
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
    /// let client = Arc::new(AptosClient::new(AptosClientType::Mainnet));
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
        client: Arc<AptosClient>,
        address_a: &str,
        address_b: &str,
        limit: Option<u64>,
        start: Option<u64>,
    ) -> Result<Vec<Transaction>, String> {
        let query = TransactionQuery { start, limit };
        let transactions = Self::get_address_transactions(client, address_a, query).await?;
        let filtered_transactions: Vec<Transaction> = transactions
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
    /// let client = Arc::new(AptosClient::new(AptosClientType::Mainnet));
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
        client: Arc<AptosClient>,
        address_a: &str, // Receiver
        address_b: &str, // Payer
        limit: Option<u64>,
        start: Option<u64>,
    ) -> Result<Vec<Transaction>, String> {
        let query = TransactionQuery { start, limit };
        let transactions =
            Self::get_address_transactions(Arc::clone(&client), address_b, query).await?;
        let filtered_transactions: Vec<Transaction> = transactions
            .into_iter()
            .filter(|txn| Self::is_transfer_from_to(txn, address_b, address_a))
            .collect();
        Ok(filtered_transactions)
    }

    /// Check if the transaction involves the specified address
    fn transaction_involves_address(transaction: &Transaction, address: &str) -> bool {
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
        transaction: &Transaction,
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
    pub fn get_user_transaction(transaction: &Transaction) -> Option<&UserTransaction> {
        match &transaction.transaction_type {
            TransactionType::UserTransaction(user_txn) => Some(user_txn),
            _ => None,
        }
    }

    /// Get transfer information in the transaction
    pub fn get_transfer_info(transaction: &Transaction) -> Option<TransferInfo> {
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
        transaction: &'a Transaction,
        event_type: &str,
    ) -> Vec<&'a Event> {
        transaction
            .events
            .iter()
            .filter(|event| event.r#type.contains(event_type))
            .collect()
    }

    /// Analyze resource changes in transactions
    pub fn analyze_resource_changes(transaction: &Transaction) -> ResourceChanges {
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
        client: Arc<AptosClient>,
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
        client: Arc<AptosClient>,
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
pub struct Transaction {
    /// The version number of the transaction (global sequence number)
    pub version: String,
    /// The hash of the transaction (unique identifier)
    pub hash: String,
    /// Hash representing the state changes caused by this transaction
    pub state_change_hash: String,
    /// Root hash of the event accumulator after this transaction
    pub event_root_hash: String,
    /// Hash of the state checkpoint (if this is a checkpoint transaction)
    pub state_checkpoint_hash: Option<String>,
    /// Amount of gas used by the transaction
    pub gas_used: String,
    /// Whether the transaction executed successfully
    pub success: bool,
    /// Status message from the virtual machine after execution
    pub vm_status: String,
    /// Root hash of the transaction accumulator
    pub accumulator_root_hash: String,
    /// List of state changes (resources modified, tables updated, etc.)
    pub changes: Vec<WriteSetChange>,
    /// Events emitted during transaction execution
    pub events: Vec<Event>,
    /// Timestamp when the transaction was executed (in microseconds)
    pub timestamp: String,
    /// Maximum gas amount that could be used for this transaction
    pub max_gas_amount: String,
    /// The specific type of transaction and its payload data
    /// Uses serde flatten to include all transaction-type-specific fields
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
    pub max_gas_amount: String,
    pub gas_unit_price: String,
    pub expiration_timestamp_secs: String,
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
    pub round: String,
    pub previous_block_votes_bitvec: Vec<u8>,
    pub proposer: String,
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
pub struct Signature {
    #[serde(rename = "type")]
    pub signature_type: String,
    pub public_key: String,
    pub signature: String,
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
    pub address: String,
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

impl Transaction {
    /// Check if the transaction was successful
    pub fn is_successful(&self) -> bool {
        self.success
    }

    /// Get transaction timestamp
    pub fn get_timestamp(&self) -> Option<u64> {
        self.timestamp.parse().ok()
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
}
