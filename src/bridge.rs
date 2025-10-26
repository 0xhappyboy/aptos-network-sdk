use crate::{AptosClient, types::ContractCall, wallet::Wallet};
use serde_json::{Value, json};
use std::sync::Arc;

/// Implementation of aptos system bridge.
pub struct SystemBridge;

impl SystemBridge {
    /// Bridging assets to other chains
    pub async fn bridge_asset(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        target_chain: &str,
        token_type: &str,
        amount: u64,
        recipient: &str,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: "0x1".to_string(),
            module_name: "bridge".to_string(),
            function_name: "transfer_to_chain".to_string(),
            type_arguments: vec![token_type.to_string()],
            arguments: vec![
                json!(target_chain),
                json!(amount.to_string()),
                json!(recipient),
            ],
        };
        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// Collect assets from other links
    pub async fn claim_bridged_asset(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        source_chain: &str,
        transaction_hash: &str,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: "0x1".to_string(),
            module_name: "bridge".to_string(),
            function_name: "claim_from_chain".to_string(),
            type_arguments: vec![],
            arguments: vec![json!(source_chain), json!(transaction_hash)],
        };
        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }
}
