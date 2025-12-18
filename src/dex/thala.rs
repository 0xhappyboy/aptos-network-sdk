/// The implementation module of Thala complete interactive logic.
use crate::{
    Aptos,
    event::EventData,
    global::mainnet::{protocol_address::THALA_PROTOCOL_ADDRESS, token_address::THL},
    types::ContractCall,
    wallet::Wallet,
};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct Thala;

impl Thala {
    /// get swap events
    pub async fn get_swap_events(client: Arc<Aptos>) -> Result<Vec<EventData>, String> {
        let event_type = format!("{}::amm::SwapEvent", THALA_PROTOCOL_ADDRESS);
        Self::get_recent_events(client, &event_type).await
    }

    async fn get_recent_events(
        client: Arc<Aptos>,
        event_type: &str,
    ) -> Result<Vec<EventData>, String> {
        let mut all_events = Vec::new();
        let mut start_seq: Option<u64> = None;
        let events = client
            .get_account_event_vec(THALA_PROTOCOL_ADDRESS, event_type, Some(100), start_seq)
            .await
            .map_err(|e| e.to_string())?;
        for event in events {
            if let Ok(sequence) = event.sequence_number.parse::<u64>() {
                let event_data = EventData {
                    event_type: event.r#type.clone(),
                    event_data: event.data.clone(),
                    sequence_number: sequence,
                    transaction_hash: "".to_string(),
                    block_height: 0,
                };
                all_events.push(event_data);
            }
        }
        Ok(all_events)
    }

    /// add liquidity
    pub async fn add_liquidity(
        client: Arc<Aptos>,
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
            module_address: THALA_PROTOCOL_ADDRESS.to_string(),
            module_name: "amm".to_string(),
            function_name: "add_liquidity".to_string(),
            type_arguments: vec![coin_x.to_string(), coin_y.to_string()],
            arguments: vec![
                json!(amount_x.to_string()),
                json!(amount_y.to_string()),
                json!(min_amount_x.to_string()),
                json!(min_amount_y.to_string()),
            ],
        };
        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// remove liquidity
    pub async fn remove_liquidity(
        client: Arc<Aptos>,
        wallet: Arc<Wallet>,
        coin_x: &str,
        coin_y: &str,
        liquidity_amount: u64,
        min_amount_x: u64,
        min_amount_y: u64,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: THALA_PROTOCOL_ADDRESS.to_string(),
            module_name: "amm".to_string(),
            function_name: "remove_liquidity".to_string(),
            type_arguments: vec![coin_x.to_string(), coin_y.to_string()],
            arguments: vec![
                json!(liquidity_amount.to_string()),
                json!(min_amount_x.to_string()),
                json!(min_amount_y.to_string()),
            ],
        };
        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// swap exact input
    pub async fn swap_exact_input(
        client: Arc<Aptos>,
        wallet: Arc<Wallet>,
        from_coin: &str,
        to_coin: &str,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: THALA_PROTOCOL_ADDRESS.to_string(),
            module_name: "router".to_string(),
            function_name: "swap_exact_input".to_string(),
            type_arguments: vec![from_coin.to_string(), to_coin.to_string()],
            arguments: vec![
                json!(amount_in.to_string()),
                json!(min_amount_out.to_string()),
            ],
        };
        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// swap exact output
    pub async fn swap_exact_output(
        client: Arc<Aptos>,
        wallet: Arc<Wallet>,
        from_coin: &str,
        to_coin: &str,
        amount_out: u64,
        max_amount_in: u64,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: THALA_PROTOCOL_ADDRESS.to_string(),
            module_name: "router".to_string(),
            function_name: "swap_exact_output".to_string(),
            type_arguments: vec![from_coin.to_string(), to_coin.to_string()],
            arguments: vec![
                json!(amount_out.to_string()),
                json!(max_amount_in.to_string()),
            ],
        };
        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// get pool info
    pub async fn get_pool_info(
        client: Arc<Aptos>,
        coin_x: &str,
        coin_y: &str,
    ) -> Result<Value, String> {
        let resource_type = format!(
            "{}::amm::Pool<{}, {}>",
            THALA_PROTOCOL_ADDRESS, coin_x, coin_y
        );
        client
            .get_account_resource(THALA_PROTOCOL_ADDRESS, &resource_type)
            .await
            .map(|opt| opt.map(|r| r.data).unwrap_or(Value::Null))
            .map_err(|e| e.to_string())
    }

    /// get thl price
    pub async fn get_thl_price(client: Arc<Aptos>) -> Result<f64, String> {
        let apt_coin = "0x1::aptos_coin::AptosCoin";
        Self::get_price(client, THL, apt_coin, 100000000).await // 1 THL
    }

    /// 获取价格
    pub async fn get_price(
        client: Arc<Aptos>,
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

    /// listen events
    pub async fn listen_events(
        client: Arc<Aptos>,
        event_sender: broadcast::Sender<EventData>,
        event_types: Vec<ThalaEventType>,
    ) -> Result<(), String> {
        for event_type in event_types {
            let client_clone = Arc::clone(&client);
            let sender_clone = event_sender.clone();
            let event_handle = event_type.get_event_handle();
            tokio::spawn(async move {
                let mut last_sequence: Option<u64> = None;
                loop {
                    if let Ok(events) = client_clone
                        .get_account_event_vec(
                            THALA_PROTOCOL_ADDRESS,
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

                                    if let Some(filtered_event) =
                                        event_type.filter_event(&event_data)
                                    {
                                        let _ = sender_clone.send(filtered_event);
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

/// thala event type
pub enum ThalaEventType {
    SwapEvent,
    MintEvent,
    BurnEvent,
    ThlStakingEvent,
}

impl ThalaEventType {
    fn get_event_handle(&self) -> String {
        match self {
            Self::SwapEvent => "swap_events".to_string(),
            Self::MintEvent => "mint_events".to_string(),
            Self::BurnEvent => "burn_events".to_string(),
            Self::ThlStakingEvent => "staking_events".to_string(),
        }
    }

    fn filter_event(&self, event_data: &EventData) -> Option<EventData> {
        match self {
            Self::SwapEvent => {
                if let (Some(coin_x), Some(coin_y)) = (
                    event_data.event_data.get("coin_x"),
                    event_data.event_data.get("coin_y"),
                ) {
                    let coin_x_str = coin_x.as_str().unwrap_or("");
                    let coin_y_str = coin_y.as_str().unwrap_or("");

                    if coin_x_str.contains("thl_coin") || coin_y_str.contains("thl_coin") {
                        return Some(event_data.clone());
                    }
                }
                if let Some(amount_in) = event_data.event_data.get("amount_in") {
                    if let Some(amount) = amount_in.as_str().and_then(|s| s.parse::<u64>().ok()) {
                        if amount > 5000000000 {
                            return Some(event_data.clone());
                        }
                    }
                }
                None
            }
            Self::ThlStakingEvent => Some(event_data.clone()),
            _ => Some(event_data.clone()),
        }
    }
}

/// thala event parser
pub struct ThalaEventParser;

impl ThalaEventParser {
    /// parse staking event
    pub fn parse_staking_event(event_data: &EventData) -> Option<ThalaStakingEvent> {
        if !event_data.event_type.contains("staking_events") {
            return None;
        }

        Some(ThalaStakingEvent {
            user: event_data
                .event_data
                .get("user")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            action: event_data
                .event_data
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            amount: event_data
                .event_data
                .get("amount")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            thl_amount: event_data
                .event_data
                .get("thl_amount")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            timestamp: event_data.block_height,
        })
    }
}

/// thala staking event
#[derive(Debug, Clone)]
pub struct ThalaStakingEvent {
    pub user: String,
    pub action: String, // "stake", "unstake", "claim"
    pub amount: u64,
    pub thl_amount: u64,
    pub timestamp: u64,
}
