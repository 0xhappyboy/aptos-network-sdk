/// Liquidswap Module
use crate::{
    AptosClient, event::EventData, global::mainnet::protocol_address::LIQUIDSWAP_PROTOCOL_ADDRESS,
    types::ContractCall, wallet::Wallet,
};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::broadcast;

const MODULE_LIQUIDITY_POOL: &str = "liquidity_pool";
const MODULE_ROUTER: &str = "router";

const FUNC_ADD_LIQUIDITY: &str = "add_liquidity";
const FUNC_REMOVE_LIQUIDITY: &str = "remove_liquidity";
const FUNC_SWAP_EXACT_INPUT: &str = "swap_exact_input";

const FUNC_SWAP_EXACT_OUTPUT: &str = "swap_exact_output";

/// Liquidswap interoperability implementation.
pub struct Liquidswap;

impl Liquidswap {
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
            module_address: LIQUIDSWAP_PROTOCOL_ADDRESS.to_string(),
            module_name: MODULE_LIQUIDITY_POOL.to_string(),
            function_name: FUNC_ADD_LIQUIDITY.to_string(),
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
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        coin_x: &str,
        coin_y: &str,
        liquidity_amount: u64,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: LIQUIDSWAP_PROTOCOL_ADDRESS.to_string(),
            module_name: MODULE_LIQUIDITY_POOL.to_string(),
            function_name: FUNC_REMOVE_LIQUIDITY.to_string(),
            type_arguments: vec![coin_x.to_string(), coin_y.to_string()],
            arguments: vec![json!(liquidity_amount.to_string())],
        };
        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    pub async fn swap_exact_input(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        from_coin: &str,
        to_coin: &str,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: LIQUIDSWAP_PROTOCOL_ADDRESS.to_string(),
            module_name: MODULE_ROUTER.to_string(),
            function_name: FUNC_SWAP_EXACT_INPUT.to_string(),
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

    pub async fn swap_exact_output(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        from_coin: &str,
        to_coin: &str,
        amount_out: u64,
        max_amount_in: u64,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: LIQUIDSWAP_PROTOCOL_ADDRESS.to_string(),
            module_name: MODULE_ROUTER.to_string(),
            function_name: FUNC_SWAP_EXACT_OUTPUT.to_string(),
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
        client: Arc<AptosClient>,
        coin_x: &str,
        coin_y: &str,
    ) -> Result<Value, String> {
        let resource_type = format!(
            "{}::liquidity_pool::LiquidityPool<{}, {}>",
            LIQUIDSWAP_PROTOCOL_ADDRESS, coin_x, coin_y
        );
        client
            .get_account_resource(LIQUIDSWAP_PROTOCOL_ADDRESS, &resource_type)
            .await
            .map(|opt| opt.map(|r| r.data).unwrap_or(Value::Null))
            .map_err(|e| e.to_string())
    }

    /// listen Liquidswap events
    pub async fn listen_events(
        client: Arc<AptosClient>,
        event_sender: broadcast::Sender<EventData>,
        event_types: Vec<LiquidswapEventType>,
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
                            LIQUIDSWAP_PROTOCOL_ADDRESS,
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

    /// get price
    pub async fn get_price(
        client: Arc<AptosClient>,
        from_coin: &str,
        to_coin: &str,
        amount: u64,
    ) -> Result<f64, String> {
        let pool_info = Self::get_pool_info(client, from_coin, to_coin).await?;
        if let (Some(reserve_x), Some(reserve_y)) = (
            pool_info.get("coin_x_reserve").and_then(|v| v.as_str()),
            pool_info.get("coin_y_reserve").and_then(|v| v.as_str()),
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
}

/// Liquidswap event type
pub enum LiquidswapEventType {
    SwapEvent,
    AddLiquidityEvent,
    RemoveLiquidityEvent,
    FlashSwapEvent,
}

impl LiquidswapEventType {
    fn get_event_handle(&self) -> String {
        match self {
            Self::SwapEvent => "swap_events".to_string(),
            Self::AddLiquidityEvent => "add_liquidity_events".to_string(),
            Self::RemoveLiquidityEvent => "remove_liquidity_events".to_string(),
            Self::FlashSwapEvent => "flash_swap_events".to_string(),
        }
    }

    fn filter_event(&self, event_data: &EventData) -> Option<EventData> {
        match self {
            Self::SwapEvent => {
                if let Some(amount_in) = event_data.event_data.get("amount_in") {
                    if let Some(amount) = amount_in.as_str().and_then(|s| s.parse::<u64>().ok()) {
                        if amount > 1000000000 {
                            return Some(event_data.clone());
                        }
                    }
                }
                None
            }
            Self::AddLiquidityEvent => {
                if let (Some(amount_x), Some(amount_y)) = (
                    event_data.event_data.get("amount_x"),
                    event_data.event_data.get("amount_y"),
                ) {
                    let amount_x_val = amount_x
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0);
                    let amount_y_val = amount_y
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0);

                    if amount_x_val > 500000000 || amount_y_val > 500000000 {
                        return Some(event_data.clone());
                    }
                }
                None
            }
            _ => Some(event_data.clone()),
        }
    }
}

/// Liquidswap event parser
pub struct LiquidswapEventParser;

impl LiquidswapEventParser {
    pub fn parse_swap_event(event_data: &EventData) -> Option<LiquidswapSwapEvent> {
        if !event_data.event_type.contains("swap_events") {
            return None;
        }
        Some(LiquidswapSwapEvent {
            sender: event_data
                .event_data
                .get("sender")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            amount_in: event_data
                .event_data
                .get("amount_in")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            amount_out: event_data
                .event_data
                .get("amount_out")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            coin_x: event_data
                .event_data
                .get("coin_x")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            coin_y: event_data
                .event_data
                .get("coin_y")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            timestamp: event_data.block_height,
        })
    }

    pub fn parse_add_liquidity_event(
        event_data: &EventData,
    ) -> Option<LiquidswapAddLiquidityEvent> {
        if !event_data.event_type.contains("add_liquidity_events") {
            return None;
        }

        Some(LiquidswapAddLiquidityEvent {
            provider: event_data
                .event_data
                .get("provider")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            amount_x: event_data
                .event_data
                .get("amount_x")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            amount_y: event_data
                .event_data
                .get("amount_y")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            liquidity_minted: event_data
                .event_data
                .get("liquidity_minted")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            coin_x: event_data
                .event_data
                .get("coin_x")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            coin_y: event_data
                .event_data
                .get("coin_y")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct LiquidswapSwapEvent {
    pub sender: String,
    pub amount_in: u64,
    pub amount_out: u64,
    pub coin_x: String,
    pub coin_y: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct LiquidswapAddLiquidityEvent {
    pub provider: String,
    pub amount_x: u64,
    pub amount_y: u64,
    pub liquidity_minted: u64,
    pub coin_x: String,
    pub coin_y: String,
}
