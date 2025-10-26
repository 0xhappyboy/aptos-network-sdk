use crate::{
    AptosClient, contract::Contract, event::EventData,
    global::mainnet::protocol_address::CELLANASWAP_PROTOCOL_ADDRESS, types::ContractCall,
    wallet::Wallet,
};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::broadcast;

const LISTEN_EVENT_TYPE: [&str; 3] = ["swap_events", "liquidity_events", "cell_farming_events"];

/// Interoperability implementation for Cellana Dex.
pub struct Cellana;

impl Cellana {
    /// add liquidity
    pub async fn add_liquidity(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        coin_x: &str,
        coin_y: &str,
        amount_x: u64,
        amount_y: u64,
        slippage: f64,
    ) -> Result<Value, String> {
        let min_amount_x = (amount_x as f64 * (1.0 - slippage)) as u64;
        let min_amount_y = (amount_y as f64 * (1.0 - slippage)) as u64;
        let contract_call = ContractCall {
            module_address: CELLANASWAP_PROTOCOL_ADDRESS.to_string(),
            module_name: "liquidity_pool".to_string(),
            function_name: "add_liquidity".to_string(),
            type_arguments: vec![coin_x.to_string(), coin_y.to_string()],
            arguments: vec![
                json!(amount_x.to_string()),
                json!(amount_y.to_string()),
                json!(min_amount_x.to_string()),
                json!(min_amount_y.to_string()),
            ],
        };
        Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// swap token
    pub async fn swap(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        from_coin: &str,
        to_coin: &str,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: CELLANASWAP_PROTOCOL_ADDRESS.to_string(),
            module_name: "router".to_string(),
            function_name: "swap".to_string(),
            type_arguments: vec![from_coin.to_string(), to_coin.to_string()],
            arguments: vec![
                json!(amount_in.to_string()),
                json!(min_amount_out.to_string()),
            ],
        };
        Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// get pool info
    pub async fn get_pool_info(
        client: Arc<AptosClient>,
        coin_x: &str,
        coin_y: &str,
    ) -> Result<Value, String> {
        let resource_type = format!(
            "{}::liquidity_pool::Pool<{}, {}>",
            CELLANASWAP_PROTOCOL_ADDRESS, coin_x, coin_y
        );
        client
            .get_account_resource(CELLANASWAP_PROTOCOL_ADDRESS, &resource_type)
            .await
            .map(|opt| opt.map(|r| r.data).unwrap_or(Value::Null))
            .map_err(|e| e.to_string())
    }

    /// get cell token price
    pub async fn get_cell_price(client: Arc<AptosClient>) -> Result<f64, String> {
        let cell_coin = format!("{}::cell_coin::CELL", CELLANASWAP_PROTOCOL_ADDRESS);
        let apt_coin = "0x1::aptos_coin::AptosCoin";

        Self::get_price(client, &cell_coin, apt_coin, 100000000).await // 1 CELL
    }

    /// get price
    pub async fn get_price(
        client: Arc<AptosClient>,
        from_coin: &str,
        to_coin: &str,
        amount: u64,
    ) -> Result<f64, String> {
        let pool_info = Self::get_pool_info(client, from_coin, to_coin).await?;
        if let (Some(reserve_x), Some(reserve_y)) = (
            pool_info.get("reserve_x").and_then(|v| v.as_str()),
            pool_info.get("reserve_y").and_then(|v| v.as_str()),
        ) {
            let reserve_x: u64 = reserve_x.parse().unwrap_or(0);
            let reserve_y: u64 = reserve_y.parse().unwrap_or(0);
            if reserve_x == 0 || reserve_y == 0 {
                return Ok(0.0);
            }
            let amount_with_fee = amount * 997;
            let numerator = amount_with_fee * reserve_y;
            let denominator = reserve_x * 1000 + amount_with_fee;
            if denominator == 0 {
                return Ok(0.0);
            }
            Ok(numerator as f64 / denominator as f64)
        } else {
            Ok(0.0)
        }
    }

    /// listen cellana event
    pub async fn listen_events(
        client: Arc<AptosClient>,
        event_sender: broadcast::Sender<EventData>,
        event_config: CellanaEventConfig,
    ) -> Result<(), String> {
        for event_handle in LISTEN_EVENT_TYPE {
            let client_clone = Arc::clone(&client);
            let sender_clone = event_sender.clone();
            let config_clone = event_config.clone();
            tokio::spawn(async move {
                let mut last_sequence: Option<u64> = None;
                loop {
                    if let Ok(events) = client_clone
                        .get_account_event_vec(
                            CELLANASWAP_PROTOCOL_ADDRESS,
                            &event_handle,
                            Some(100),
                            last_sequence,
                        )
                        .await
                    {
                        for event in events {
                            if let Ok(sequence) = event.sequence_number.parse::<u64>() {
                                if last_sequence.map(|last| sequence > last).unwrap_or(true) {
                                    let event_data = EventData {
                                        event_type: event.r#type.clone(),
                                        event_data: event.data.clone(),
                                        sequence_number: sequence,
                                        transaction_hash: "".to_string(),
                                        block_height: client_clone
                                            .get_chain_height()
                                            .await
                                            .unwrap_or(0)
                                            as u64,
                                    };
                                    if CellanaEventFilter::should_include(
                                        &event_data,
                                        &config_clone,
                                    ) {
                                        let _ = sender_clone.send(event_data);
                                    }

                                    last_sequence = Some(sequence);
                                }
                            }
                        }
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            });
        }
        Ok(())
    }
}

/// cellana event config
#[derive(Debug, Clone)]
pub struct CellanaEventConfig {
    pub monitor_cell_pairs: bool,
    pub min_swap_amount: u64,
    pub monitor_farming: bool,
    pub tracked_tokens: Vec<String>,
}

/// cellana event filter
pub struct CellanaEventFilter;

impl CellanaEventFilter {
    pub fn should_include(event_data: &EventData, config: &CellanaEventConfig) -> bool {
        if event_data.event_type.contains("swap_events") {
            Self::filter_swap_event(event_data, config)
        } else if event_data.event_type.contains("cell_farming_events") {
            config.monitor_farming
        } else {
            true
        }
    }

    fn filter_swap_event(event_data: &EventData, config: &CellanaEventConfig) -> bool {
        if let Some(amount_in) = event_data.event_data.get("amount_in") {
            if let Some(amount) = amount_in.as_str().and_then(|s| s.parse::<u64>().ok()) {
                if amount < config.min_swap_amount {
                    return false;
                }
            }
        }
        if config.monitor_cell_pairs {
            if let (Some(coin_x), Some(coin_y)) = (
                event_data.event_data.get("coin_x"),
                event_data.event_data.get("coin_y"),
            ) {
                let coin_x_str = coin_x.as_str().unwrap_or("");
                let coin_y_str = coin_y.as_str().unwrap_or("");

                if coin_x_str.contains("cell_coin") || coin_y_str.contains("cell_coin") {
                    return true;
                }
            }
        }
        if !config.tracked_tokens.is_empty() {
            if let (Some(coin_x), Some(coin_y)) = (
                event_data.event_data.get("coin_x"),
                event_data.event_data.get("coin_y"),
            ) {
                let coin_x_str = coin_x.as_str().unwrap_or("");
                let coin_y_str = coin_y.as_str().unwrap_or("");
                if !config
                    .tracked_tokens
                    .iter()
                    .any(|t| t == coin_x_str || t == coin_y_str)
                {
                    return false;
                }
            }
        }
        true
    }
}

/// cellana farming
pub struct CellanaFarming;

impl CellanaFarming {
    /// stake lp
    pub async fn stake_lp(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        pool_id: u64,
        amount: u64,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: CELLANASWAP_PROTOCOL_ADDRESS.to_string(),
            module_name: "farming".to_string(),
            function_name: "stake".to_string(),
            type_arguments: vec![],
            arguments: vec![json!(pool_id.to_string()), json!(amount.to_string())],
        };
        Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// harvest rewards
    pub async fn harvest_rewards(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        pool_id: u64,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: CELLANASWAP_PROTOCOL_ADDRESS.to_string(),
            module_name: "farming".to_string(),
            function_name: "harvest".to_string(),
            type_arguments: vec![],
            arguments: vec![json!(pool_id.to_string())],
        };

        Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }
}
