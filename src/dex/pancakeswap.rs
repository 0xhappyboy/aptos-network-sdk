/// The implementation module of pancakeswap complete interactive logic.
use crate::{
    Aptos,
    event::EventData,
    global::mainnet::{
        protocol_address::PANCAKESWAP_FACTORY_PROTOCOL_ADDRESS,
        token_address::{APT, CAKE},
    },
    types::ContractCall,
    wallet::Wallet,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct PancakeSwap;

impl PancakeSwap {
    /// get swap events
    pub async fn get_swap_events(client: Arc<Aptos>) -> Result<Vec<EventData>, String> {
        let event_type = format!("{}::swap::SwapEvent", PANCAKESWAP_FACTORY_PROTOCOL_ADDRESS);
        Self::get_recent_events(client, &event_type).await
    }

    async fn get_recent_events(
        client: Arc<Aptos>,
        event_type: &str,
    ) -> Result<Vec<EventData>, String> {
        let mut all_events = Vec::new();
        let mut start_seq: Option<u64> = None;

        let events = client
            .get_account_event_vec(
                PANCAKESWAP_FACTORY_PROTOCOL_ADDRESS,
                event_type,
                Some(100),
                start_seq,
            )
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
        coin_a: &str,
        coin_b: &str,
        amount_a_desired: u64,
        amount_b_desired: u64,
        amount_a_min: u64,
        amount_b_min: u64,
        to: &str,
        deadline: u64,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: PANCAKESWAP_FACTORY_PROTOCOL_ADDRESS.to_string(),
            module_name: "router".to_string(),
            function_name: "add_liquidity".to_string(),
            type_arguments: vec![coin_a.to_string(), coin_b.to_string()],
            arguments: vec![
                json!(amount_a_desired.to_string()),
                json!(amount_b_desired.to_string()),
                json!(amount_a_min.to_string()),
                json!(amount_b_min.to_string()),
                json!(to),
                json!(deadline.to_string()),
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
        coin_a: &str,
        coin_b: &str,
        liquidity: u64,
        amount_a_min: u64,
        amount_b_min: u64,
        to: &str,
        deadline: u64,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: PANCAKESWAP_FACTORY_PROTOCOL_ADDRESS.to_string(),
            module_name: "router".to_string(),
            function_name: "remove_liquidity".to_string(),
            type_arguments: vec![coin_a.to_string(), coin_b.to_string()],
            arguments: vec![
                json!(liquidity.to_string()),
                json!(amount_a_min.to_string()),
                json!(amount_b_min.to_string()),
                json!(to),
                json!(deadline.to_string()),
            ],
        };
        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// swap exact tokens for tokens
    pub async fn swap_exact_tokens_for_tokens(
        client: Arc<Aptos>,
        wallet: Arc<Wallet>,
        amount_in: u64,
        amount_out_min: u64,
        path: Vec<&str>,
        to: &str,
        deadline: u64,
    ) -> Result<Value, String> {
        let type_arguments: Vec<String> = path.iter().map(|s| s.to_string()).collect();
        let path_values: Vec<Value> = path.iter().map(|s| json!(s)).collect();
        let contract_call = ContractCall {
            module_address: PANCAKESWAP_FACTORY_PROTOCOL_ADDRESS.to_string(),
            module_name: "router".to_string(),
            function_name: "swap_exact_tokens_for_tokens".to_string(),
            type_arguments,
            arguments: vec![
                json!(amount_in.to_string()),
                json!(amount_out_min.to_string()),
                json!(path_values),
                json!(to),
                json!(deadline.to_string()),
            ],
        };
        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// get reserves
    pub async fn get_reserves(
        client: Arc<Aptos>,
        coin_a: &str,
        coin_b: &str,
    ) -> Result<(u64, u64), String> {
        let pair_address = Self::get_pair_address(coin_a, coin_b);
        let resource_type = format!(
            "{}::swap::TokenPairReserve<{}, {}>",
            PANCAKESWAP_FACTORY_PROTOCOL_ADDRESS, coin_a, coin_b
        );
        match client
            .get_account_resource(&pair_address, &resource_type)
            .await
        {
            Ok(Some(resource)) => {
                let reserve_a = resource
                    .data
                    .get("reserve0")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let reserve_b = resource
                    .data
                    .get("reserve1")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                Ok((reserve_a, reserve_b))
            }
            Ok(None) => Ok((0, 0)),
            Err(e) => Err(e),
        }
    }

    /// get pair address
    pub fn get_pair_address(coin_a: &str, coin_b: &str) -> String {
        let (token_x, token_y) = if coin_a < coin_b {
            (coin_a, coin_b)
        } else {
            (coin_b, coin_a)
        };
        let factory_address = PANCAKESWAP_FACTORY_PROTOCOL_ADDRESS;
        let salt = "pancake_swap_pair";
        let mut hasher = Sha256::new();
        hasher.update(factory_address.as_bytes());
        hasher.update("::factory::Pair".as_bytes());
        hasher.update(token_x.as_bytes());
        hasher.update(token_y.as_bytes());
        hasher.update(salt.as_bytes());
        let hash = hasher.finalize();
        let mut addr_bytes = [0u8; 32];
        addr_bytes.copy_from_slice(&hash[..32]);
        let mut result = String::with_capacity(66);
        result.push_str("0x");
        for byte in addr_bytes {
            result.push_str(&format!("{:02x}", byte));
        }
        result
    }

    /// get cake price
    pub async fn get_cake_price(client: Arc<Aptos>) -> Result<f64, String> {
        let (reserve_cake, reserve_apt) = Self::get_reserves(client, CAKE, APT).await?;
        if reserve_cake == 0 {
            return Ok(0.0);
        }
        Ok(reserve_apt as f64 / reserve_cake as f64)
    }

    /// listen events
    pub async fn listen_events(
        client: Arc<Aptos>,
        event_sender: broadcast::Sender<EventData>,
        filters: PancakeSwapEventFilters,
    ) -> Result<(), String> {
        let event_handles = vec![
            "swap_events".to_string(),
            "mint_events".to_string(),
            "burn_events".to_string(),
            "sync_events".to_string(),
        ];
        for event_handle in event_handles {
            let client_clone = Arc::clone(&client);
            let sender_clone = event_sender.clone();
            let filters_clone = filters.clone();
            tokio::spawn(async move {
                let mut last_sequence: Option<u64> = None;
                loop {
                    if let Ok(events) = client_clone
                        .get_account_event_vec(
                            PANCAKESWAP_FACTORY_PROTOCOL_ADDRESS,
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

                                    if PancakeSwapEventFilter::apply_filters(
                                        &event_data,
                                        &filters_clone,
                                    ) {
                                        let _ = sender_clone.send(event_data);
                                    }

                                    last_sequence = Some(sequence);
                                }
                            }
                        }
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                }
            });
        }
        Ok(())
    }
}

/// Pancake Swap Event Filters
#[derive(Debug, Clone)]
pub struct PancakeSwapEventFilters {
    pub min_swap_amount: Option<u64>,
    pub include_cake_pairs: bool,
    pub tracked_pairs: Option<Vec<(String, String)>>,
}

///  Pancake Swap Event Filter
pub struct PancakeSwapEventFilter;

impl PancakeSwapEventFilter {
    pub fn apply_filters(event_data: &EventData, filters: &PancakeSwapEventFilters) -> bool {
        if event_data.event_type.contains("swap_events") {
            Self::filter_swap_events(event_data, filters)
        } else if event_data.event_type.contains("mint_events") {
            Self::filter_mint_events(event_data, filters)
        } else {
            true
        }
    }

    fn filter_swap_events(event_data: &EventData, filters: &PancakeSwapEventFilters) -> bool {
        if let Some(min_amount) = filters.min_swap_amount {
            if let (Some(amount0_in), Some(amount1_in)) = (
                event_data.event_data.get("amount0_in"),
                event_data.event_data.get("amount1_in"),
            ) {
                let amount0 = amount0_in
                    .as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
                let amount1 = amount1_in
                    .as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);

                if amount0 < min_amount && amount1 < min_amount {
                    return false;
                }
            }
        }

        if filters.include_cake_pairs {
            if let (Some(token0), Some(token1)) = (
                event_data.event_data.get("token0"),
                event_data.event_data.get("token1"),
            ) {
                let token0_str = token0.as_str().unwrap_or("");
                let token1_str = token1.as_str().unwrap_or("");

                if token0_str.contains("CakeOFT") || token1_str.contains("CakeOFT") {
                    return true;
                }
            }
        }

        if let Some(tracked_pairs) = &filters.tracked_pairs {
            if let (Some(token0), Some(token1)) = (
                event_data.event_data.get("token0"),
                event_data.event_data.get("token1"),
            ) {
                let token0_str = token0.as_str().unwrap_or("");
                let token1_str = token1.as_str().unwrap_or("");

                if !tracked_pairs.iter().any(|(t0, t1)| {
                    (t0 == token0_str && t1 == token1_str) || (t0 == token1_str && t1 == token0_str)
                }) {
                    return false;
                }
            }
        }

        true
    }

    fn filter_mint_events(_event_data: &EventData, _filters: &PancakeSwapEventFilters) -> bool {
        // Receive all liquidity addition events
        true
    }
}
