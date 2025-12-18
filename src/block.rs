use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{Aptos, trade::TransactionInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub block_height: String,
    pub block_hash: String,
    #[serde(rename = "block_timestamp")]
    pub timestamp: String,
    pub first_version: String,
    pub last_version: String,
    #[serde(default)]
    pub transactions: Option<Vec<TransactionInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    /// Block height
    pub block_height: u64,
    /// Block hash
    pub block_hash: String,
    /// First version in block
    pub first_version: u64,
    /// Last version in block
    pub last_version: u64,
    /// Block timestamp in microseconds since epoch
    pub timestamp: u64,
    /// Transaction count in block
    pub transaction_count: usize,
    /// Block transactions
    pub transactions: Option<Vec<TransactionInfo>>,
}

impl BlockInfo {
    /// Convert from Aptos Block
    pub fn from_aptos_block(block: &Block) -> Self {
        let transaction_count = block.transactions.as_ref().map_or(0, |v| v.len());

        Self {
            block_height: block.block_height.parse::<u64>().unwrap_or(0),
            block_hash: block.block_hash.clone(),
            first_version: block.first_version.parse::<u64>().unwrap_or(0),
            last_version: block.last_version.parse::<u64>().unwrap_or(0),
            timestamp: block.timestamp.parse::<u64>().unwrap_or(0),
            transaction_count,
            transactions: None,
        }
    }

    /// Convert from Aptos Block with full transactions
    pub fn from_aptos_block_with_txs(block: &Block, transactions: Vec<TransactionInfo>) -> Self {
        let transaction_count = transactions.len();

        Self {
            block_height: block.block_height.parse::<u64>().unwrap_or(0),
            block_hash: block.block_hash.clone(),
            first_version: block.first_version.parse::<u64>().unwrap_or(0),
            last_version: block.last_version.parse::<u64>().unwrap_or(0),
            timestamp: block.timestamp.parse::<u64>().unwrap_or(0),
            transaction_count,
            transactions: Some(transactions),
        }
    }

    /// Get block timestamp in seconds
    pub fn timestamp_seconds(&self) -> f64 {
        self.timestamp as f64 / 1_000_000.0
    }

    /// Get block timestamp in milliseconds
    pub fn timestamp_millis(&self) -> u64 {
        self.timestamp / 1000
    }

    /// Get transactions per second (TPS) for this block
    pub fn tps(&self) -> Option<f64> {
        if self.timestamp > 0 {
            let duration_seconds = self.timestamp_seconds();
            if duration_seconds > 0.0 {
                return Some(self.transaction_count as f64 / duration_seconds);
            }
        }
        None
    }

    /// Get block time in seconds
    pub fn block_time_seconds(&self) -> Option<f64> {
        if self.timestamp > 0 {
            Some(self.timestamp_seconds())
        } else {
            None
        }
    }

    /// Check if block contains transactions
    pub fn has_transactions(&self) -> bool {
        self.transaction_count > 0
    }

    /// Get transaction version range
    pub fn transaction_version_range(&self) -> (u64, u64) {
        (self.first_version, self.last_version)
    }

    /// Calculate block size (approximation)
    pub fn estimated_size(&self) -> usize {
        // Basic estimation: 100 bytes per transaction + base block size
        let base_size = 500; // Base block metadata size
        base_size + (self.transaction_count * 100)
    }
}
#[cfg(test)]
mod tests {
    use crate::AptosType;

    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_latest_block() {
        let aptos = Arc::new(Aptos::new(AptosType::Testnet));
        let chain_height_result = aptos.get_chain_height().await;
        match chain_height_result {
            Ok(height) => {
                println!("Chain height: {}", height);
                match aptos.get_block_by_height(height).await {
                    Ok(block) => {
                        let block_info = BlockInfo::from_aptos_block(&block);
                        println!("✅ Successfully got latest block");
                        println!("   Block height: {}", block_info.block_height);
                        println!("   Block hash: {}", block_info.block_hash);
                        println!("   Timestamp: {}", block_info.timestamp);
                        println!("   Transaction count: {}", block_info.transaction_count);
                        // Test conversion functions
                        println!("   Timestamp (seconds): {}", block_info.timestamp_seconds());
                        println!("   Timestamp (millis): {}", block_info.timestamp_millis());
                        if let Some(tps) = block_info.tps() {
                            println!("   Estimated TPS: {:.2}", tps);
                        }
                        if let Some(block_time) = block_info.block_time_seconds() {
                            println!("   Block time: {:.2} seconds", block_time);
                        }
                        println!("   Estimated size: {} bytes", block_info.estimated_size());
                        println!(
                            "   Transaction range: {:?}",
                            block_info.transaction_version_range()
                        );
                        println!("   Has transactions: {}", block_info.has_transactions());
                    }
                    Err(e) => {
                        println!("❌ Failed to get block by height: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to get chain height: {}", e);
            }
        }
    }
}
