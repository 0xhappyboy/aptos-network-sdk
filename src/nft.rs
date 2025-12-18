/// Implementation of NFT function for aptos 0x3 system library.
use crate::{
    Aptos,
    global::mainnet::{
        sys_address::X_3,
        sys_module::{
            self,
            token::{
                collections, create_collection_script, create_token_script, token_store,
                transfer_script,
            },
        },
    },
    types::ContractCall,
    wallet::Wallet,
};
use serde_json::{Value, json};
use std::sync::Arc;

pub struct NFTManager;

impl NFTManager {
    /// create nft collection
    pub async fn create_nft_collection(
        client: Arc<Aptos>,
        wallet: Arc<Wallet>,
        name: &str,
        description: &str,
        uri: &str,
        max_amount: Option<u64>,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: X_3.to_string(),
            module_name: sys_module::token::name.to_string(),
            function_name: create_collection_script.to_string(),
            type_arguments: vec![],
            arguments: vec![
                json!(name),
                json!(description),
                json!(uri),
                json!(max_amount.unwrap_or(u64::MAX).to_string()),
                json!(false), // mutable
            ],
        };
        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// create nft
    pub async fn create_nft(
        client: Arc<Aptos>,
        wallet: Arc<Wallet>,
        collection: &str,
        name: &str,
        description: &str,
        supply: u64,
        uri: &str,
        royalty_points_per_million: u64,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: X_3.to_string(),
            module_name: sys_module::token::name.to_string(),
            function_name: create_token_script.to_string(),
            type_arguments: vec![],
            arguments: vec![
                json!(collection),
                json!(name),
                json!(description),
                json!(supply.to_string()),
                json!(supply.to_string()), // max supply
                json!(uri),
                json!(wallet.address().map_err(|e| e.to_string())?), // royalty payee
                json!(royalty_points_per_million.to_string()),
                json!(0u64.to_string()),     // royalty denominator
                json!(vec![] as Vec<Value>), // property keys
                json!(vec![] as Vec<Value>), // property values
                json!(vec![] as Vec<Value>), // property types
            ],
        };
        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// transfer nft
    pub async fn transfer_nft(
        client: Arc<Aptos>,
        wallet: Arc<Wallet>,
        token_id: &str,
        recipient: &str,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: X_3.to_string(),
            module_name: sys_module::token::name.to_string(),
            function_name: transfer_script.to_string(),
            type_arguments: vec![],
            arguments: vec![
                json!(recipient),
                json!(token_id),
                json!(1u64.to_string()), // amount
            ],
        };
        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// get nft balance
    pub async fn get_nft_balance(
        client: Arc<Aptos>,
        address: &str,
        token_id: &str,
    ) -> Result<u64, String> {
        let resource_type = format!("{}", token_store);
        if let Some(resource) = client.get_account_resource(address, &resource_type).await? {
            Ok(resource
                .data
                .get("tokens")
                .and_then(|t| t.as_object())
                .and_then(|t| t.get(token_id))
                .and_then(|t| t.get("amount"))
                .and_then(|a| a.as_str())
                .and_then(|a| a.parse().ok())
                .unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    /// get nft metedata
    pub async fn get_nft_metedata(
        client: Arc<Aptos>,
        creator: &str,
        collection: &str,
        name: &str,
    ) -> Result<Value, String> {
        let resource_type = format!("{}", collections);
        if let Some(resource) = client.get_account_resource(creator, &resource_type).await? {
            Ok(resource
                .data
                .get("token_data")
                .cloned()
                .unwrap_or(Value::Null))
        } else {
            Ok(Value::Null)
        }
    }
}
