use crate::{
    AptosClient, event::EventData, global::mainnet::protocol_address::ANIMESWAP_PROTOCOL_ADDRESS,
    types::ContractCall, wallet::Wallet,
};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Implementation of interoperability functions for AnimeSwap.
pub struct AnimeSwap;

impl AnimeSwap {
    /// get swap event
    pub async fn get_swap_events(client: Arc<AptosClient>) -> Result<Vec<EventData>, String> {
        let event_type = format!("{}::swap::SwapEvent", ANIMESWAP_PROTOCOL_ADDRESS);
        Self::get_events_by_time_range(client, &event_type).await
    }
    async fn get_events_by_time_range(
        client: Arc<AptosClient>,
        event_type: &str,
    ) -> Result<Vec<EventData>, String> {
        let mut all_events = Vec::new();
        let mut start_seq: Option<u64> = None;
        loop {
            let events = client
                .get_account_event_vec(ANIMESWAP_PROTOCOL_ADDRESS, event_type, Some(100), start_seq)
                .await
                .map_err(|e| e.to_string())?;
            let events_count = events.len();
            if events.is_empty() {
                break;
            }
            for event in events {
                if let Ok(sequence) = event.sequence_number.parse::<u64>() {
                    let event_timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let event_data = EventData {
                        event_type: event.r#type.clone(),
                        event_data: event.data.clone(),
                        sequence_number: sequence,
                        transaction_hash: "".to_string(),
                        block_height: 0,
                    };
                    all_events.push(event_data);
                    start_seq = Some(sequence);
                }
            }
            if events_count < 100 {
                break;
            }
        }
        Ok(all_events)
    }
    /// add liquidity
    pub async fn add_liquidity(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        coin_a: &str,
        coin_b: &str,
        amount_a: u64,
        amount_b: u64,
        slippage: f64,
    ) -> Result<Value, String> {
        let min_amount_a = (amount_a as f64 * (1.0 - slippage)) as u64;
        let min_amount_b = (amount_b as f64 * (1.0 - slippage)) as u64;

        let contract_call = ContractCall {
            module_address: ANIMESWAP_PROTOCOL_ADDRESS.to_string(),
            module_name: "router".to_string(),
            function_name: "add_liquidity".to_string(),
            type_arguments: vec![coin_a.to_string(), coin_b.to_string()],
            arguments: vec![
                json!(amount_a.to_string()),
                json!(amount_b.to_string()),
                json!(min_amount_a.to_string()),
                json!(min_amount_b.to_string()),
            ],
        };

        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// swap exact tokens for tokens
    pub async fn swap_exact_tokens_for_tokens(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        path: Vec<&str>,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<Value, String> {
        if path.len() < 2 {
            return Err("Path must contain at least 2 tokens".to_string());
        }
        let type_arguments: Vec<String> = path.iter().map(|s| s.to_string()).collect();
        let path_arguments: Vec<Value> = path.iter().map(|s| json!(s)).collect();
        let contract_call = ContractCall {
            module_address: ANIMESWAP_PROTOCOL_ADDRESS.to_string(),
            module_name: "router".to_string(),
            function_name: "swap_exact_tokens_for_tokens".to_string(),
            type_arguments,
            arguments: vec![
                json!(amount_in.to_string()),
                json!(min_amount_out.to_string()),
                json!(path_arguments),
            ],
        };
        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// get reserves
    pub async fn get_reserves(
        client: Arc<AptosClient>,
        coin_a: &str,
        coin_b: &str,
    ) -> Result<(u64, u64), String> {
        let resource_type = format!(
            "{}::swap::TokenPairReserve<{}, {}>",
            ANIMESWAP_PROTOCOL_ADDRESS, coin_a, coin_b
        );
        match client
            .get_account_resource(ANIMESWAP_PROTOCOL_ADDRESS, &resource_type)
            .await
        {
            Ok(Some(resource)) => {
                let reserve_a = resource
                    .data
                    .get("reserve_a")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let reserve_b = resource
                    .data
                    .get("reserve_b")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                Ok((reserve_a, reserve_b))
            }
            Ok(None) => Ok((0, 0)),
            Err(e) => Err(e),
        }
    }

    /// listen events
    pub async fn listen_events(
        client: Arc<AptosClient>,
        event_sender: broadcast::Sender<EventData>,
        filters: AnimeSwapEventFilters,
    ) -> Result<(), String> {
        let event_types = vec![
            "swap_events".to_string(),
            "mint_events".to_string(),
            "burn_events".to_string(),
        ];
        for event_type in event_types {
            let client_clone = Arc::clone(&client);
            let sender_clone = event_sender.clone();
            let filters_clone = filters.clone();
            tokio::spawn(async move {
                let mut last_sequence: Option<u64> = None;
                loop {
                    if let Ok(events) = client_clone
                        .get_account_event_vec(
                            ANIMESWAP_PROTOCOL_ADDRESS,
                            &event_type,
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
                                    if AnimeSwapEventFilter::apply_filters(
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

/// anime swap event filters
#[derive(Debug, Clone)]
pub struct AnimeSwapEventFilters {
    pub min_swap_amount: Option<u64>,
    pub tracked_tokens: Option<Vec<String>>,
    pub min_liquidity_amount: Option<u64>,
}

/// anime swap event filter
pub struct AnimeSwapEventFilter;

impl AnimeSwapEventFilter {
    pub fn apply_filters(event_data: &EventData, filters: &AnimeSwapEventFilters) -> bool {
        if event_data.event_type.contains("swap_events") {
            Self::filter_swap_events(event_data, filters)
        } else if event_data.event_type.contains("mint_events") {
            Self::filter_mint_events(event_data, filters)
        } else if event_data.event_type.contains("burn_events") {
            Self::filter_burn_events(event_data, filters)
        } else {
            true
        }
    }

    fn filter_swap_events(event_data: &EventData, filters: &AnimeSwapEventFilters) -> bool {
        if let Some(min_amount) = filters.min_swap_amount {
            if let Some(amount_in) = event_data
                .event_data
                .get("amount0_in")
                .or_else(|| event_data.event_data.get("amount1_in"))
            {
                if let Some(amount) = amount_in.as_str().and_then(|s| s.parse::<u64>().ok()) {
                    if amount < min_amount {
                        return false;
                    }
                }
            }
        }

        if let Some(tracked_tokens) = &filters.tracked_tokens {
            if let (Some(token0), Some(token1)) = (
                event_data.event_data.get("token0"),
                event_data.event_data.get("token1"),
            ) {
                let token0_str = token0.as_str().unwrap_or("");
                let token1_str = token1.as_str().unwrap_or("");
                if !tracked_tokens
                    .iter()
                    .any(|t| t == token0_str || t == token1_str)
                {
                    return false;
                }
            }
        }
        true
    }

    fn filter_mint_events(event_data: &EventData, filters: &AnimeSwapEventFilters) -> bool {
        if let Some(min_amount) = filters.min_liquidity_amount {
            if let Some(amount0) = event_data.event_data.get("amount0") {
                if let Some(amount) = amount0.as_str().and_then(|s| s.parse::<u64>().ok()) {
                    if amount < min_amount {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn filter_burn_events(_event_data: &EventData, _filters: &AnimeSwapEventFilters) -> bool {
        // to be realized
        true
    }
}

/// animeSwap Price Calculator
pub struct AnimeSwapPriceCalculator;

impl AnimeSwapPriceCalculator {
    /// find the best transaction path
    pub async fn find_best_path(
        client: Arc<AptosClient>,
        from_token: &str,
        to_token: &str,
        amount_in: u64,
        intermediate_tokens: Vec<&str>,
    ) -> Result<(Vec<String>, u64), String> {
        let mut best_path = vec![from_token.to_string(), to_token.to_string()];
        let mut best_output = 0u64;
        if let Ok((reserve_in, reserve_out)) =
            AnimeSwap::get_reserves(Arc::clone(&client), from_token, to_token).await
        {
            let direct_output = Self::calculate_output_amount(amount_in, reserve_in, reserve_out);
            if direct_output > best_output {
                best_output = direct_output;
            }
        }
        for intermediate in intermediate_tokens {
            let path1 = vec![from_token, intermediate];
            let path2 = vec![intermediate, to_token];
            if let (Ok((reserve1_in, reserve1_out)), Ok((reserve2_in, reserve2_out))) = (
                AnimeSwap::get_reserves(Arc::clone(&client), path1[0], path1[1]).await,
                AnimeSwap::get_reserves(Arc::clone(&client), path2[0], path2[1]).await,
            ) {
                let output1 = Self::calculate_output_amount(amount_in, reserve1_in, reserve1_out);
                let final_output =
                    Self::calculate_output_amount(output1, reserve2_in, reserve2_out);

                if final_output > best_output {
                    best_output = final_output;
                    best_path = vec![
                        from_token.to_string(),
                        intermediate.to_string(),
                        to_token.to_string(),
                    ];
                }
            }
        }
        Ok((best_path, best_output))
    }

    fn calculate_output_amount(amount_in: u64, reserve_in: u64, reserve_out: u64) -> u64 {
        if reserve_in == 0 || reserve_out == 0 {
            return 0;
        }
        let amount_in_with_fee = amount_in * 997;
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = reserve_in * 1000 + amount_in_with_fee;
        numerator / denominator
    }
}
