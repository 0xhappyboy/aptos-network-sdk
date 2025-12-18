use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::trade::TransactionInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub sequence_number: String,
    pub authentication_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub r#type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub bytecode: String,
    pub abi: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainInfo {
    pub chain_id: u8,
    pub epoch: String,
    pub ledger_version: String,
    pub ledger_timestamp: String,
    pub node_role: String,
    pub block_height: String,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Transaction {
//     pub version: Option<String>,
//     pub hash: String,
//     pub sender: String,
//     pub sequence_number: String,
//     pub max_gas_amount: String,
//     pub gas_unit_price: String,
//     pub expiration_timestamp_secs: String,
//     pub payload: serde_json::Value,
//     pub signature: Option<serde_json::Value>,
//     pub events: Vec<Event>,
//     pub timestamp: String,
//     pub r#type: String,
//     pub success: bool,
//     pub vm_status: String,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub guid: serde_json::Value,
    pub sequence_number: String,
    pub r#type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasEstimation {
    pub deprioritized_gas_estimate: Option<u64>,
    pub gas_estimate: u64,
    pub prioritized_gas_estimate: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ViewRequest {
    pub function: String,
    pub type_arguments: Vec<String>,
    pub arguments: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TableRequest {
    pub key_type: String,
    pub value_type: String,
    pub key: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCall {
    pub module_address: String,
    pub module_name: String,
    pub function_name: String,
    pub type_arguments: Vec<String>,
    pub arguments: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractReadResult {
    pub success: bool,
    pub data: Value,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractWriteResult {
    pub success: bool,
    pub transaction_hash: String,
    pub gas_used: String,
    pub events: Vec<Value>,
    pub error: Option<String>,
}

impl ContractWriteResult {
    pub fn gas_used_as_u64(&self) -> u64 {
        self.gas_used.parse::<u64>().unwrap_or(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEvent {
    pub event_type: String,
    pub data: HashMap<String, Value>,
    pub sequence_number: String,
}

#[derive(serde::Serialize)]
pub struct EntryFunctionPayload {
    pub module_address: Vec<u8>,
    pub module_name: Vec<u8>,
    pub function_name: Vec<u8>,
    pub type_arguments: Vec<Vec<u8>>,
    pub arguments: Vec<Vec<u8>>,
}

#[derive(serde::Serialize)]
pub struct RawTransactionForSigning {
    pub sender: Vec<u8>,
    pub sequence_number: u64,
    pub payload: Vec<u8>,
    pub max_gas_amount: u64,
    pub gas_unit_price: u64,
    pub expiration_timestamp_secs: u64,
    pub chain_id: u8,
}
