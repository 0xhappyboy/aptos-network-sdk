use crate::{Aptos, types::ContractCall, wallet::Wallet};
use serde_json::{Value, json};
use std::sync::Arc;

/// Implementation of the staking function for the aptos system.
pub struct SystemStaking;

impl SystemStaking {
    /// stake $apt
    pub async fn stake(
        client: Arc<Aptos>,
        wallet: Arc<Wallet>,
        amount: u64,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: "0x1".to_string(),
            module_name: "staking_contract".to_string(),
            function_name: "stake".to_string(),
            type_arguments: vec![],
            arguments: vec![json!(amount.to_string())],
        };
        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// unstake
    pub async fn unstake(
        client: Arc<Aptos>,
        wallet: Arc<Wallet>,
        amount: u64,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: "0x1".to_string(),
            module_name: "staking_contract".to_string(),
            function_name: "unstake".to_string(),
            type_arguments: vec![],
            arguments: vec![json!(amount.to_string())],
        };
        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// claim staking rewards
    pub async fn claim(client: Arc<Aptos>, wallet: Arc<Wallet>) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: "0x1".to_string(),
            module_name: "staking_contract".to_string(),
            function_name: "claim_rewards".to_string(),
            type_arguments: vec![],
            arguments: vec![],
        };
        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// get staking info
    pub async fn get_staking_info(
        client: Arc<Aptos>,
        address: &str,
    ) -> Result<Value, String> {
        let resource_type = "0x1::staking_contract::StakingInfo";
        client
            .get_account_resource(address, resource_type)
            .await
            .map(|opt| opt.map(|r| r.data).unwrap_or(Value::Null))
            .map_err(|e| e.to_string())
    }
}
