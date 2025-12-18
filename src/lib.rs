pub mod block;
pub mod bridge;
pub mod contract;
pub mod dex;
pub mod event;
pub mod global;
pub mod multicall;
pub mod nft;
pub mod nft_market;
pub mod staking;
pub mod token;
pub mod tool;
pub mod trade;
pub mod types;
pub mod wallet;

use crate::{
    block::Block,
    global::rpc::{APTOS_DEVNET_URL, APTOS_MAINNET_URL, APTOS_TESTNET_URL},
    trade::TransactionInfo,
    types::*,
};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

/// waiting transaction delay time
const WAITING_TRANSACTION_DELAY_TIME: u64 = 500;

/// client type
#[derive(Debug, Clone)]
pub enum AptosType {
    Mainnet,
    Testnet,
    Devnet,
}

#[derive(Debug, Clone)]
pub struct Aptos {
    client: Client,
    base_url: String,
}

impl Aptos {
    pub fn new(network: AptosType) -> Self {
        let base_url = match network {
            AptosType::Mainnet => APTOS_MAINNET_URL.to_string(),
            AptosType::Testnet => APTOS_TESTNET_URL.to_string(),
            AptosType::Devnet => APTOS_DEVNET_URL.to_string(),
        };
        Aptos {
            client: Client::new(),
            base_url,
        }
    }

    /// get chain height
    pub async fn get_chain_height(&self) -> Result<u64, String> {
        let chain_info = self.get_chain_info().await?;
        Ok(chain_info.block_height.parse::<u64>().unwrap_or(0))
    }

    /// get chain height
    pub async fn get_ledger_version(&self) -> Result<u64, String> {
        let chain_info = self.get_chain_info().await?;
        Ok(chain_info.ledger_version.parse::<u64>().unwrap_or(0))
    }

    /// get account info
    pub async fn get_account_info(&self, address: &str) -> Result<AccountInfo, String> {
        let url: String = format!("{}/accounts/{}", self.base_url, address);
        let response = self.client.get(&url).send().await.unwrap();
        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap();
            return Err(format!("api error: {}", error_msg).to_string());
        }

        let account_info: AccountInfo = response.json().await.unwrap();
        Ok(account_info)
    }

    /// get account resources vec
    pub async fn get_account_resource_vec(&self, address: &str) -> Result<Vec<Resource>, String> {
        let url = format!("{}/accounts/{}/resources", self.base_url, address);
        let response = self.client.get(&url).send().await.unwrap();
        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap();
            return Err(format!("api error: {}", error_msg).to_string());
        }
        let resources: Vec<Resource> = response.json().await.unwrap();
        Ok(resources)
    }

    /// get account resource
    pub async fn get_account_resource(
        &self,
        address: &str,
        resource_type: &str,
    ) -> Result<Option<Resource>, String> {
        let url = format!(
            "{}/accounts/{}/resource/{}",
            self.base_url, address, resource_type
        );
        let response = self.client.get(&url).send().await.unwrap();

        if response.status() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap();
            return Err(format!("api error: {}", error_msg).to_string());
        }

        let resource: Resource = response.json().await.unwrap();
        Ok(Some(resource))
    }

    /// get account module vec
    pub async fn get_account_module_vec(&self, address: &str) -> Result<Vec<Module>, String> {
        let url = format!("{}/accounts/{}/modules", self.base_url, address);
        let response = self.client.get(&url).send().await.unwrap();
        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap();
            return Err(format!("api error: {}", error_msg).to_string());
        }
        let modules: Vec<Module> = response.json().await.unwrap();
        Ok(modules)
    }

    /// get account module
    pub async fn get_account_module(
        &self,
        address: &str,
        module_name: &str,
    ) -> Result<Option<Module>, String> {
        let url = format!(
            "{}/accounts/{}/module/{}",
            self.base_url, address, module_name
        );
        let response = self.client.get(&url).send().await.unwrap();
        if response.status() == 404 {
            return Ok(None);
        }
        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap();
            return Err(format!("api error: {}", error_msg).to_string());
        }
        let module: Module = response.json().await.unwrap();
        Ok(Some(module))
    }

    /// submit transaction
    pub async fn submit_transaction(&self, txn_payload: &Value) -> Result<TransactionInfo, String> {
        let url = format!("{}/transactions", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(txn_payload)
            .send()
            .await
            .unwrap();
        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap();
            return Err(format!("transaction submit failed: {}", error_msg).to_string());
        }
        let transaction: TransactionInfo = response.json().await.unwrap();
        Ok(transaction)
    }

    /// get transaction info
    pub async fn get_transaction_info_by_hash(
        &self,
        tx_hash: &str,
    ) -> Result<TransactionInfo, String> {
        let url = format!("{}/transactions/by_hash/{}", self.base_url, tx_hash);
        let response = self.client.get(&url).send().await.unwrap();
        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap();
            return Err(format!("api error: {}", error_msg).to_string());
        }
        let transaction: TransactionInfo = response
            .json()
            .await
            .map_err(|e| format!("transaction parsing error: {:?}", e))?;
        Ok(transaction)
    }

    /// get transaction by version
    pub async fn get_transaction_info_by_version(
        &self,
        version: u64,
    ) -> Result<TransactionInfo, String> {
        let url = format!("{}/transactions/by_version/{}", self.base_url, version);
        let response = self.client.get(&url).send().await.unwrap();
        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap();
            return Err(format!("api error: {}", error_msg).to_string());
        }
        let transaction: TransactionInfo = response.json().await.unwrap();
        Ok(transaction)
    }

    /// get account transaction vec
    pub async fn get_account_transaction_vec(
        &self,
        address: &str,
        limit: Option<u64>,
        start: Option<u64>,
    ) -> Result<Vec<TransactionInfo>, String> {
        let limit = limit.unwrap_or(25);
        let mut url = format!(
            "{}/accounts/{}/transactions?limit={}",
            self.base_url, address, limit
        );
        if let Some(start) = start {
            url.push_str(&format!("&start={}", start));
        }
        let response = self.client.get(&url).send().await.unwrap();
        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap();
            return Err(format!("api error: {}", error_msg).to_string());
        }
        let transactions: Vec<TransactionInfo> = response.json().await.unwrap();
        Ok(transactions)
    }

    /// get chain info
    pub async fn get_chain_info(&self) -> Result<ChainInfo, String> {
        let url = format!("{}/", self.base_url);
        let response = self.client.get(&url).send().await.unwrap();
        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap();
            return Err(format!("api error: {}", error_msg).to_string());
        }
        let ledger_info: ChainInfo = response.json().await.unwrap();
        Ok(ledger_info)
    }

    /// get block by height
    pub async fn get_block_by_height(&self, height: u64) -> Result<Block, String> {
        let url = format!("{}/blocks/by_height/{}", self.base_url, height);
        let response = self.client.get(&url).send().await.unwrap();
        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap();
            return Err(format!("api error: {}", error_msg).to_string());
        }
        let block: Block = response.json().await.unwrap();
        Ok(block)
    }

    /// get block by version
    pub async fn get_block_by_version(&self, version: u64) -> Result<Block, String> {
        let url = format!("{}/blocks/by_version/{}", self.base_url, version);
        let response = self.client.get(&url).send().await.unwrap();
        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap();
            return Err(format!("api error: {}", error_msg).to_string());
        }
        let block: Block = response.json().await.unwrap();
        Ok(block)
    }

    /// get account event vec
    pub async fn get_account_event_vec(
        &self,
        address: &str,
        event_type: &str,
        limit: Option<u64>,
        start: Option<u64>,
    ) -> Result<Vec<Event>, String> {
        let limit = limit.unwrap_or(25);
        let mut url = format!(
            "{}/accounts/{}/events/{}?limit={}",
            self.base_url, address, event_type, limit
        );
        if let Some(start) = start {
            url.push_str(&format!("&start={}", start));
        }
        let response = self.client.get(&url).send().await.unwrap();
        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap();
            return Err(format!("api error: {}", error_msg).to_string());
        }
        let events: Vec<Event> = response.json().await.unwrap();
        Ok(events)
    }

    /// get table item
    pub async fn get_table_item(
        &self,
        table_handle: &str,
        key_type: &str,
        value_type: &str,
        key: &Value,
    ) -> Result<Value, String> {
        let url = format!("{}/tables/{}/item", self.base_url, table_handle);
        let request = TableRequest {
            key_type: key_type.to_string(),
            value_type: value_type.to_string(),
            key: key.clone(),
        };
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .unwrap();
        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap();
            return Err(format!("api error: {}", error_msg).to_string());
        }
        let value: Value = response.json().await.unwrap();
        Ok(value)
    }

    /// view function
    pub async fn view(&self, view_request: &ViewRequest) -> Result<Vec<Value>, String> {
        let url = format!("{}/view", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(view_request)
            .send()
            .await
            .unwrap();
        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap();
            return Err(format!("api error: {}", error_msg).to_string());
        }
        let result: Vec<Value> = response.json().await.unwrap();
        Ok(result)
    }

    /// estimate gas price
    pub async fn estimate_gas_price(&self) -> Result<u64, String> {
        let url = format!("{}/estimate_gas_price", self.base_url);
        let response = self.client.get(&url).send().await.unwrap();
        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap();
            return Err(format!("api error: {}", error_msg).to_string());
        }
        let gas_estimation: GasEstimation = response.json().await.unwrap();
        Ok(gas_estimation.gas_estimate * 2000)
    }

    /// get account balance
    pub async fn get_account_balance(&self, address: &str) -> Result<u64, String> {
        let resources = self.get_account_resource_vec(address).await.unwrap();
        for resource in resources {
            if resource.r#type == "0x1::coin::CoinStore<0x1::aptos_coin::AptosCoin>" {
                if let Some(data) = resource.data.as_object() {
                    if let Some(coin) = data.get("coin") {
                        if let Some(value) = coin.get("value") {
                            return if let Some(balance) = value.as_str() {
                                Ok(balance.parse().unwrap_or(0))
                            } else if let Some(balance) = value.as_u64() {
                                Ok(balance)
                            } else {
                                Ok(0)
                            };
                        }
                    }
                }
            }
        }
        Ok(0)
    }
    /// get token balance
    pub async fn get_token_balance(&self, address: &str, token_type: &str) -> Result<u64, String> {
        let resource_type = format!("0x1::coin::CoinStore<{}>", token_type);
        if let Some(resource) = self
            .get_account_resource(address, &resource_type)
            .await
            .unwrap()
        {
            if let Some(data) = resource.data.as_object() {
                if let Some(coin) = data.get("coin") {
                    if let Some(value) = coin.get("value") {
                        return if let Some(balance) = value.as_str() {
                            Ok(balance.parse().unwrap_or(0))
                        } else if let Some(balance) = value.as_u64() {
                            Ok(balance)
                        } else {
                            Ok(0)
                        };
                    }
                }
            }
        }
        Ok(0)
    }
    /// waiting transaction
    pub async fn waiting_transaction(
        &self,
        txn_hash: &str,
        timeout_secs: u64,
    ) -> Result<TransactionInfo, String> {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);
        while start.elapsed() < timeout {
            match self.get_transaction_info_by_hash(txn_hash).await {
                Ok(txn) => {
                    // transaction completed
                    return Ok(txn);
                }
                Err(e) => {
                    // during transaction processing, delay accessing the transaction status again.
                    tokio::time::sleep(Duration::from_millis(WAITING_TRANSACTION_DELAY_TIME)).await;
                }
            }
        }
        Err(format!(
            "Transaction timeout tx:{:?}\ntime:{:?}",
            txn_hash, timeout_secs
        )
        .to_string())
    }
    /// determine whether the transaction is successful
    pub async fn is_transaction_successful(&self, txn_hash: &str) -> Result<bool, String> {
        match self.get_transaction_info_by_hash(txn_hash).await {
            Ok(t) => Ok(t.success),
            Err(e) => Err(e),
        }
    }
    /// get apt balance by account
    pub async fn get_apt_balance_by_account(&self, address: &str) -> Result<f64, String> {
        match self.get_account_balance(address).await {
            Ok(balance) => Ok(balance as f64 / 100_000_000.0),
            Err(e) => Err(e),
        }
    }
    /// get account sequence number
    pub async fn get_account_sequence_number(&self, address: &str) -> Result<u64, String> {
        match self.get_account_info(address).await {
            Ok(info) => Ok(info.sequence_number.parse::<u64>().unwrap()),
            Err(e) => Err(e),
        }
    }
    /// account exists
    pub async fn account_exists(&self, address: &str) -> Result<bool, String> {
        match self.get_account_info(address).await {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.to_string().contains("Account not found") {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_specific_transaction() {
        let client = Aptos::new(AptosType::Mainnet);
        let known_tx_hash = "0xc4da6f117be28bdf63ee455dcb845fe2c4447c5b89a9fb20e3afa92d9b8f2f50";
        let result = client.get_transaction_info_by_hash(known_tx_hash).await;
        match result {
            Ok(tx) => {
                println!("✅Find Transaction: {:?}", tx);
                println!("Hash: {}", tx.hash);
                println!("Version: {}", tx.version);
            }
            Err(e) => {
                println!("❌ error: {}", e);
            }
        }
    }
}
