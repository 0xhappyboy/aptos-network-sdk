use crate::{
    AptosClient, event::EventData, global::mainnet::protocol_address::AUXSWAP_PROTOCOL_ADDRESS,
    types::ContractCall, wallet::Wallet,
};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Implementation of Aux Exchange AMM functions.
pub struct AuxExchange;

impl AuxExchange {
    /// listen Aux Exchange events
    pub async fn listen_events(
        client: Arc<AptosClient>,
        event_sender: broadcast::Sender<EventData>,
        event_types: Vec<AuxEventType>,
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
                            AUXSWAP_PROTOCOL_ADDRESS,
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
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            });
        }
        Ok(())
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
            module_address: AUXSWAP_PROTOCOL_ADDRESS.to_string(),
            module_name: "amm".to_string(),
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

    pub async fn swap_exact_output(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        from_coin: &str,
        to_coin: &str,
        max_amount_in: u64,
        amount_out: u64,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: AUXSWAP_PROTOCOL_ADDRESS.to_string(),
            module_name: "amm".to_string(),
            function_name: "swap_exact_output".to_string(),
            type_arguments: vec![from_coin.to_string(), to_coin.to_string()],
            arguments: vec![
                json!(max_amount_in.to_string()),
                json!(amount_out.to_string()),
            ],
        };

        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// add liquidity
    pub async fn add_liquidity(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        coin_a: &str,
        coin_b: &str,
        amount_a: u64,
        amount_b: u64,
        min_lp_amount: u64,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: AUXSWAP_PROTOCOL_ADDRESS.to_string(),
            module_name: "amm".to_string(),
            function_name: "add_liquidity".to_string(),
            type_arguments: vec![coin_a.to_string(), coin_b.to_string()],
            arguments: vec![
                json!(amount_a.to_string()),
                json!(amount_b.to_string()),
                json!(min_lp_amount.to_string()),
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
        coin_a: &str,
        coin_b: &str,
        lp_amount: u64,
        min_amount_a: u64,
        min_amount_b: u64,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: AUXSWAP_PROTOCOL_ADDRESS.to_string(),
            module_name: "amm".to_string(),
            function_name: "remove_liquidity".to_string(),
            type_arguments: vec![coin_a.to_string(), coin_b.to_string()],
            arguments: vec![
                json!(lp_amount.to_string()),
                json!(min_amount_a.to_string()),
                json!(min_amount_b.to_string()),
            ],
        };

        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// get pool info
    pub async fn get_pool_info(
        client: Arc<AptosClient>,
        coin_a: &str,
        coin_b: &str,
    ) -> Result<Value, String> {
        let resource_type = format!(
            "{}::amm::Pool<{}, {}>",
            AUXSWAP_PROTOCOL_ADDRESS, coin_a, coin_b
        );

        client
            .get_account_resource(AUXSWAP_PROTOCOL_ADDRESS, &resource_type)
            .await
            .map(|opt| opt.map(|r| r.data).unwrap_or(Value::Null))
            .map_err(|e| e.to_string())
    }

    /// get price
    pub async fn get_price(
        client: Arc<AptosClient>,
        from_coin: &str,
        to_coin: &str,
        amount: u64,
    ) -> Result<u64, String> {
        let pool_info = Self::get_pool_info(client, from_coin, to_coin).await?;
        if let (Some(reserve_a), Some(reserve_b)) = (
            pool_info
                .get("coin_a_reserve")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u64>().ok()),
            pool_info
                .get("coin_b_reserve")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u64>().ok()),
        ) {
            let amount_out = (amount * reserve_b) / (reserve_a + amount);
            Ok(amount_out)
        } else {
            Err("Failed to calculate price".to_string())
        }
    }

    /// get user liquidity
    pub async fn get_user_liquidity(
        client: Arc<AptosClient>,
        user_address: &str,
        coin_a: &str,
        coin_b: &str,
    ) -> Result<Value, String> {
        let resource_type = format!(
            "{}::amm::LPToken<{}, {}>",
            AUXSWAP_PROTOCOL_ADDRESS, coin_a, coin_b
        );

        client
            .get_account_resource(user_address, &resource_type)
            .await
            .map(|opt| opt.map(|r| r.data).unwrap_or(Value::Null))
            .map_err(|e| e.to_string())
    }
}

pub enum AuxEventType {
    Swap,
    AddLiquidity,
    RemoveLiquidity,
}

impl AuxEventType {
    fn get_event_handle(&self) -> String {
        match self {
            Self::Swap => "swap_events".to_string(),
            Self::AddLiquidity => "add_liquidity_events".to_string(),
            Self::RemoveLiquidity => "remove_liquidity_events".to_string(),
        }
    }

    fn filter_event(&self, event_data: &EventData) -> Option<EventData> {
        match self {
            Self::Swap => {
                if let Some(amount_in) = event_data.event_data.get("amount_in") {
                    if let Some(amount) = amount_in.as_str().and_then(|s| s.parse::<u64>().ok()) {
                        if amount > 5000000000 {
                            return Some(event_data.clone());
                        }
                    }
                }
                None
            }
            _ => Some(event_data.clone()),
        }
    }
}

pub struct AuxEventParser;

impl AuxEventParser {
    pub fn parse_swap_event(event_data: &EventData) -> Option<AuxSwapEvent> {
        if !event_data.event_type.contains("swap_events") {
            return None;
        }
        Some(AuxSwapEvent {
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
            from_coin: event_data
                .event_data
                .get("from_coin")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            to_coin: event_data
                .event_data
                .get("to_coin")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct AuxSwapEvent {
    pub sender: String,
    pub amount_in: u64,
    pub amount_out: u64,
    pub from_coin: String,
    pub to_coin: String,
}
