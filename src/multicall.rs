// src/multicall.rs
use crate::{trade::BatchTradeHandle, types::ContractCall, wallet::Wallet, AptosClient};
use futures::future::join_all;
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Semaphore;

/// Multi-contract call manager
pub struct MultiContractCall;

impl MultiContractCall {
    /// execute multiple read-only calls
    pub async fn aggregate_read(
        client: Arc<AptosClient>,
        calls: Vec<ContractCall>,
    ) -> Result<Vec<Value>, String> {
        let mut results = Vec::new();
        for call in calls {
            match crate::contract::Contract::read(Arc::clone(&client), &call).await {
                Ok(result) => results.push(json!(result)),
                Err(e) => results.push(json!({
                    "success": false,
                    "error": e
                })),
            }
        }
        Ok(results)
    }

    /// Contract call sequence with dependencies
    pub async fn execute_sequence(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        calls: Vec<(ContractCall, Option<String>)>,
    ) -> Result<Vec<Value>, String> {
        let mut results = Vec::new();
        let mut previous_result: Option<Value> = None;
        for (call, dependency) in calls {
            let mut final_call = call;
            // If there is a dependency, extract data from the previous result
            if let (Some(dep_field), Some(prev_result)) = (dependency, &previous_result) {
                if let Some(dep_value) = prev_result.get(&dep_field) {
                    // Adding dependent values ​​to parameters
                    final_call.arguments.push(dep_value.clone());
                }
            }
            match crate::contract::Contract::write(
                Arc::clone(&client),
                Arc::clone(&wallet),
                final_call,
            )
            .await
            {
                Ok(result) => {
                    let result_value = json!(result);
                    previous_result = Some(result_value.clone());
                    results.push(result_value);
                }
                Err(e) => {
                    results.push(json!({
                        "success": false,
                        "error": e
                    }));
                    break;
                }
            }
        }
        Ok(results)
    }

    /// Conditional execution execute the call only if a condition is met
    pub async fn conditional_execute(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        condition_call: ContractCall,
        execute_call: ContractCall,
    ) -> Result<Option<Value>, String> {
        // First check the conditions
        let condition_result =
            crate::contract::Contract::read(Arc::clone(&client), &condition_call).await?;
        if condition_result.success {
            // If the conditions are met, execute the call
            crate::contract::Contract::write(client, wallet, execute_call)
                .await
                .map(|result| Some(json!(result)))
                .map_err(|e| e.to_string())
        } else {
            // If the condition is not met
            Ok(None)
        }
    }

    /// Execute multiple write calls in parallel (no dependencies)
    pub async fn parallel_execute(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        calls: Vec<ContractCall>,
        max_concurrency: usize,
    ) -> Result<Vec<Value>, String> {
        BatchTradeHandle::process_batch(client, wallet, calls, max_concurrency).await
    }
}

/// Multiple call tool functions
pub struct MultiCallUtils;

impl MultiCallUtils {
    /// Creating standard multi-call parameters
    pub fn create_multicall_calls(
        base_calls: Vec<ContractCall>,
        common_args: Vec<Value>,
    ) -> Vec<ContractCall> {
        base_calls
            .into_iter()
            .map(|mut call| {
                call.arguments.extend(common_args.clone());
                call
            })
            .collect()
    }

    /// Analyzing multiple call results
    pub fn analyze_multicall_results(results: &[Value]) -> HashMap<String, usize> {
        let mut analysis = HashMap::new();
        let mut success_count = 0;
        let mut failure_count = 0;
        for result in results {
            if let Some(success) = result.get("success").and_then(|s| s.as_bool()) {
                if success {
                    success_count += 1;
                } else {
                    failure_count += 1;
                }
            }
        }
        analysis.insert("total".to_string(), results.len());
        analysis.insert("success".to_string(), success_count);
        analysis.insert("failed".to_string(), failure_count);
        analysis
    }

    /// Filter successful call results
    pub fn filter_successful_results(results: Vec<Value>) -> Vec<Value> {
        results
            .into_iter()
            .filter(|result| {
                result
                    .get("success")
                    .and_then(|s| s.as_bool())
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Extract all transaction hashes
    pub fn extract_transaction_hashes(results: &[Value]) -> Vec<String> {
        results
            .iter()
            .filter_map(|result| {
                if result
                    .get("success")
                    .and_then(|s| s.as_bool())
                    .unwrap_or(false)
                {
                    result
                        .get("transaction_hash")
                        .and_then(|h| h.as_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect()
    }
}
