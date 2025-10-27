use crate::global::mainnet::nft_market::{
    AUX_EXCHANGE, BLUEMOVE, MERCATO, PANCAKE_SWAP_NFT, SOUFFL3, TOPAZ, TRADEPORT, WAPAL,
};
// nft_marketplace.rs
use crate::{AptosClient, types::ContractCall, wallet::Wallet};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;

/// NFT marketplace aggregator manager
pub struct NFTMarketplaceAggregator;

/// Supported NFT marketplace list
pub struct Marketplaces;

impl Marketplaces {
    /// Get all marketplace addresses
    pub fn all_markets() -> Vec<&'static str> {
        vec![
            TOPAZ,
            SOUFFL3,
            BLUEMOVE,
            MERCATO,
            AUX_EXCHANGE,
            PANCAKE_SWAP_NFT,
            TRADEPORT,
            WAPAL,
        ]
    }
}

/// NFT listing information
#[derive(Debug, Clone)]
pub struct NFTListing {
    pub token_id: String,
    pub price: u64,
    pub marketplace: String,
    pub seller: String,
    pub listing_time: u64,
    pub currency: String, // Usually "0x1::aptos_coin::AptosCoin"
    pub marketplace_name: String,
}

/// Marketplace order book
#[derive(Debug, Clone)]
pub struct NFTOrderBook {
    pub token_id: String,
    pub listings: Vec<NFTListing>,
    pub best_offer: Option<u64>,  // Best offer price
    pub floor_price: Option<u64>, // Floor price
}

/// Transaction result
#[derive(Debug, Clone)]
pub struct NFTPurchaseResult {
    pub success: bool,
    pub transaction_hash: String,
    pub marketplace: String,
    pub total_cost: u64,
    pub gas_used: u64,
}

impl NFTMarketplaceAggregator {
    /// Search NFT listings across all marketplaces
    pub async fn search_nft_listings(
        client: Arc<AptosClient>,
        token_id: &str,
    ) -> Result<Vec<NFTListing>, String> {
        let mut all_listings = Vec::new();
        for marketplace in Marketplaces::all_markets() {
            if let Ok(listings) =
                Self::get_marketplace_listings(Arc::clone(&client), marketplace, token_id).await
            {
                all_listings.extend(listings);
            }
        }
        // Sort by price
        all_listings.sort_by(|a, b| a.price.cmp(&b.price));
        Ok(all_listings)
    }

    /// Get NFT listings from specific marketplace
    async fn get_marketplace_listings(
        client: Arc<AptosClient>,
        marketplace_address: &str,
        token_id: &str,
    ) -> Result<Vec<NFTListing>, String> {
        let mut listings = Vec::new();
        // Call different parsing logic based on marketplace address
        match marketplace_address {
            TOPAZ => {
                listings = Self::parse_topaz_listings(client, token_id).await?;
            }
            SOUFFL3 => {
                listings = Self::parse_souffl3_listings(client, token_id).await?;
            }
            BLUEMOVE => {
                listings = Self::parse_bluemove_listings(client, token_id).await?;
            }
            MERCATO => {
                listings = Self::parse_mercato_listings(client, token_id).await?;
            }
            AUX_EXCHANGE => {
                listings = Self::parse_aux_listings(client, token_id).await?;
            }
            PANCAKE_SWAP_NFT => {
                listings = Self::parse_pancake_listings(client, token_id).await?;
            }
            TRADEPORT => {
                listings = Self::parse_tradeport_listings(client, token_id).await?;
            }
            WAPAL => {
                listings = Self::parse_wapal_listings(client, token_id).await?;
            }
            _ => {}
        }
        Ok(listings)
    }

    /// Parse Topaz marketplace listings
    async fn parse_topaz_listings(
        client: Arc<AptosClient>,
        token_id: &str,
    ) -> Result<Vec<NFTListing>, String> {
        let mut listings = Vec::new();
        // Topaz uses specific listing resource structure
        if let Ok(resources) = client.get_account_resource_vec(TOPAZ).await {
            for resource in resources {
                if resource.r#type.contains("::listings::") {
                    if let Some(listing) =
                        Self::extract_listing_from_resource(&resource, token_id, "Topaz")
                    {
                        listings.push(listing);
                    }
                }
            }
        }
        Ok(listings)
    }

    /// Parse Souffl3 marketplace listings
    async fn parse_souffl3_listings(
        client: Arc<AptosClient>,
        token_id: &str,
    ) -> Result<Vec<NFTListing>, String> {
        let mut listings = Vec::new();
        // Souffl3 specific resource structure
        if let Ok(resources) = client.get_account_resource_vec(SOUFFL3).await {
            for resource in resources {
                if resource.r#type.contains("::market::") || resource.r#type.contains("::listing::")
                {
                    if let Some(listing) =
                        Self::extract_listing_from_resource(&resource, token_id, "Souffl3")
                    {
                        listings.push(listing);
                    }
                }
            }
        }

        Ok(listings)
    }

    /// Parse BlueMove marketplace listings
    async fn parse_bluemove_listings(
        client: Arc<AptosClient>,
        token_id: &str,
    ) -> Result<Vec<NFTListing>, String> {
        let mut listings = Vec::new();
        if let Ok(resources) = client.get_account_resource_vec(BLUEMOVE).await {
            for resource in resources {
                if resource.r#type.contains("::Marketplace") {
                    if let Some(listing) =
                        Self::extract_listing_from_resource(&resource, token_id, "BlueMove")
                    {
                        listings.push(listing);
                    }
                }
            }
        }
        Ok(listings)
    }

    /// Parse Mercato marketplace listings
    async fn parse_mercato_listings(
        client: Arc<AptosClient>,
        token_id: &str,
    ) -> Result<Vec<NFTListing>, String> {
        let mut listings = Vec::new();

        if let Ok(resources) = client.get_account_resource_vec(MERCATO).await {
            for resource in resources {
                if resource.r#type.contains("::market") {
                    if let Some(listing) =
                        Self::extract_listing_from_resource(&resource, token_id, "Mercato")
                    {
                        listings.push(listing);
                    }
                }
            }
        }

        Ok(listings)
    }

    /// Parse AUX Exchange listings
    async fn parse_aux_listings(
        client: Arc<AptosClient>,
        token_id: &str,
    ) -> Result<Vec<NFTListing>, String> {
        let mut listings = Vec::new();

        if let Ok(resources) = client.get_account_resource_vec(AUX_EXCHANGE).await {
            for resource in resources {
                if resource.r#type.contains("::amm::") || resource.r#type.contains("::clob::") {
                    if let Some(listing) =
                        Self::extract_listing_from_resource(&resource, token_id, "AUX")
                    {
                        listings.push(listing);
                    }
                }
            }
        }

        Ok(listings)
    }

    /// Parse PancakeSwap NFT listings
    async fn parse_pancake_listings(
        client: Arc<AptosClient>,
        token_id: &str,
    ) -> Result<Vec<NFTListing>, String> {
        let mut listings = Vec::new();

        if let Ok(resources) = client.get_account_resource_vec(PANCAKE_SWAP_NFT).await {
            for resource in resources {
                if resource.r#type.contains("::nft_market") {
                    if let Some(listing) =
                        Self::extract_listing_from_resource(&resource, token_id, "PancakeSwap")
                    {
                        listings.push(listing);
                    }
                }
            }
        }

        Ok(listings)
    }

    /// Parse Tradeport marketplace listings
    async fn parse_tradeport_listings(
        client: Arc<AptosClient>,
        token_id: &str,
    ) -> Result<Vec<NFTListing>, String> {
        let mut listings = Vec::new();

        if let Ok(resources) = client.get_account_resource_vec(TRADEPORT).await {
            for resource in resources {
                if resource.r#type.contains("::marketplace")
                    || resource.r#type.contains("::listing")
                {
                    if let Some(listing) = Self::extract_tradeport_listing_from_resource(
                        &resource,
                        token_id,
                        "Tradeport",
                    ) {
                        listings.push(listing);
                    }
                }
            }
        }

        Ok(listings)
    }

    /// Parse Wapal marketplace listings
    async fn parse_wapal_listings(
        client: Arc<AptosClient>,
        token_id: &str,
    ) -> Result<Vec<NFTListing>, String> {
        let mut listings = Vec::new();

        if let Ok(resources) = client.get_account_resource_vec(WAPAL).await {
            for resource in resources {
                if resource.r#type.contains("::wapal") || resource.r#type.contains("::market") {
                    if let Some(listing) =
                        Self::extract_wapal_listing_from_resource(&resource, token_id, "Wapal")
                    {
                        listings.push(listing);
                    }
                }
            }
        }

        Ok(listings)
    }

    /// Extract listing information from resource data
    fn extract_listing_from_resource(
        resource: &crate::types::Resource,
        token_id: &str,
        marketplace_name: &str,
    ) -> Option<NFTListing> {
        if let Value::Object(data) = &resource.data {
            // Try to extract price information from different marketplace data structures
            let price = data
                .get("price")
                .or_else(|| data.get("list_price"))
                .or_else(|| data.get("amount"))
                .and_then(|p| p.as_str())
                .and_then(|p| p.parse::<u64>().ok())
                .unwrap_or(0);

            let seller = data
                .get("seller")
                .or_else(|| data.get("owner"))
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();

            if price > 0 {
                return Some(NFTListing {
                    token_id: token_id.to_string(),
                    price,
                    marketplace: resource.r#type.clone(),
                    seller,
                    listing_time: 0, // Needs to be parsed from data
                    currency: "0x1::aptos_coin::AptosCoin".to_string(),
                    marketplace_name: marketplace_name.to_string(),
                });
            }
        }
        None
    }

    /// Extract listing information from Tradeport resource data
    fn extract_tradeport_listing_from_resource(
        resource: &crate::types::Resource,
        token_id: &str,
        marketplace_name: &str,
    ) -> Option<NFTListing> {
        if let Value::Object(data) = &resource.data {
            // Tradeport specific data structure
            let price = data
                .get("price")
                .or_else(|| data.get("list_price"))
                .or_else(|| data.get("buy_now_price"))
                .and_then(|p| p.as_str())
                .and_then(|p| p.parse::<u64>().ok())
                .unwrap_or(0);

            let seller = data
                .get("seller")
                .or_else(|| data.get("owner"))
                .or_else(|| data.get("creator"))
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();

            if price > 0 {
                return Some(NFTListing {
                    token_id: token_id.to_string(),
                    price,
                    marketplace: resource.r#type.clone(),
                    seller,
                    listing_time: data
                        .get("created_at")
                        .and_then(|t| t.as_str())
                        .and_then(|t| t.parse::<u64>().ok())
                        .unwrap_or(0),
                    currency: "0x1::aptos_coin::AptosCoin".to_string(),
                    marketplace_name: marketplace_name.to_string(),
                });
            }
        }
        None
    }

    /// Extract listing information from Wapal resource data
    fn extract_wapal_listing_from_resource(
        resource: &crate::types::Resource,
        token_id: &str,
        marketplace_name: &str,
    ) -> Option<NFTListing> {
        if let Value::Object(data) = &resource.data {
            // Wapal specific data structure
            let price = data
                .get("price")
                .or_else(|| data.get("amount"))
                .or_else(|| data.get("sale_price"))
                .and_then(|p| p.as_str())
                .and_then(|p| p.parse::<u64>().ok())
                .unwrap_or(0);

            let seller = data
                .get("seller")
                .or_else(|| data.get("owner"))
                .or_else(|| data.get("current_owner"))
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();

            if price > 0 {
                return Some(NFTListing {
                    token_id: token_id.to_string(),
                    price,
                    marketplace: resource.r#type.clone(),
                    seller,
                    listing_time: data
                        .get("list_time")
                        .and_then(|t| t.as_str())
                        .and_then(|t| t.parse::<u64>().ok())
                        .unwrap_or(0),
                    currency: "0x1::aptos_coin::AptosCoin".to_string(),
                    marketplace_name: marketplace_name.to_string(),
                });
            }
        }
        None
    }

    /// Get best price (cross-market comparison)
    pub async fn get_best_price(
        client: Arc<AptosClient>,
        token_id: &str,
    ) -> Result<Option<NFTListing>, String> {
        let listings = Self::search_nft_listings(client, token_id).await?;
        Ok(listings.into_iter().min_by_key(|listing| listing.price))
    }

    /// Purchase NFT on specified marketplace
    pub async fn purchase_nft(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        listing: &NFTListing,
    ) -> Result<NFTPurchaseResult, String> {
        let contract_call = Self::build_purchase_call(listing)?;

        match crate::contract::Contract::write(client, wallet, contract_call).await {
            Ok(result) => Ok(NFTPurchaseResult {
                success: true,
                transaction_hash: result.transaction_hash.clone(),
                marketplace: listing.marketplace_name.clone(),
                total_cost: listing.price,
                gas_used: result.gas_used_as_u64(),
            }),
            Err(e) => Err(e),
        }
    }

    /// Build purchase call
    fn build_purchase_call(listing: &NFTListing) -> Result<ContractCall, String> {
        let (module_address, module_name, function_name, arguments) =
            match listing.marketplace_name.as_str() {
                "Topaz" => Self::build_topaz_purchase_call(listing),
                "Souffl3" => Self::build_souffl3_purchase_call(listing),
                "BlueMove" => Self::build_bluemove_purchase_call(listing),
                "Mercato" => Self::build_mercato_purchase_call(listing),
                "AUX" => Self::build_aux_purchase_call(listing),
                "PancakeSwap" => Self::build_pancake_purchase_call(listing),
                "Tradeport" => Self::build_tradeport_purchase_call(listing),
                "Wapal" => Self::build_wapal_purchase_call(listing),
                _ => return Err("Unsupported marketplace".to_string()),
            };

        Ok(ContractCall {
            module_address,
            module_name,
            function_name,
            type_arguments: vec![],
            arguments,
        })
    }

    /// Build Topaz purchase call
    fn build_topaz_purchase_call(listing: &NFTListing) -> (String, String, String, Vec<Value>) {
        (
            TOPAZ.to_string(),
            "marketplace".to_string(),
            "purchase".to_string(),
            vec![
                json!(listing.token_id),
                json!(listing.seller),
                json!(listing.price.to_string()),
            ],
        )
    }

    /// Build Souffl3 purchase call
    fn build_souffl3_purchase_call(listing: &NFTListing) -> (String, String, String, Vec<Value>) {
        (
            SOUFFL3.to_string(),
            "market".to_string(),
            "buy".to_string(),
            vec![json!(listing.token_id), json!(listing.price.to_string())],
        )
    }

    /// Build BlueMove purchase call
    fn build_bluemove_purchase_call(listing: &NFTListing) -> (String, String, String, Vec<Value>) {
        (
            BLUEMOVE.to_string(),
            "marketplace".to_string(),
            "buy_token".to_string(),
            vec![json!(listing.token_id), json!(listing.price.to_string())],
        )
    }

    /// Build Mercato purchase call
    fn build_mercato_purchase_call(listing: &NFTListing) -> (String, String, String, Vec<Value>) {
        (
            MERCATO.to_string(),
            "market".to_string(),
            "purchase".to_string(),
            vec![json!(listing.token_id), json!(listing.price.to_string())],
        )
    }

    /// Build AUX purchase call
    fn build_aux_purchase_call(listing: &NFTListing) -> (String, String, String, Vec<Value>) {
        (
            AUX_EXCHANGE.to_string(),
            "nft_market".to_string(),
            "buy".to_string(),
            vec![json!(listing.token_id), json!(listing.price.to_string())],
        )
    }

    /// Build PancakeSwap purchase call
    fn build_pancake_purchase_call(listing: &NFTListing) -> (String, String, String, Vec<Value>) {
        (
            PANCAKE_SWAP_NFT.to_string(),
            "nft_market".to_string(),
            "purchase".to_string(),
            vec![json!(listing.token_id), json!(listing.price.to_string())],
        )
    }

    /// Build Tradeport purchase call
    fn build_tradeport_purchase_call(listing: &NFTListing) -> (String, String, String, Vec<Value>) {
        (
            TRADEPORT.to_string(),
            "marketplace".to_string(),
            "purchase".to_string(),
            vec![
                json!(listing.token_id),
                json!(listing.seller),
                json!(listing.price.to_string()),
            ],
        )
    }

    /// Build Wapal purchase call
    fn build_wapal_purchase_call(listing: &NFTListing) -> (String, String, String, Vec<Value>) {
        (
            WAPAL.to_string(),
            "market".to_string(),
            "buy_nft".to_string(),
            vec![json!(listing.token_id), json!(listing.price.to_string())],
        )
    }

    /// List NFT on multiple marketplaces
    pub async fn list_nft_on_markets(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        token_id: &str,
        price: u64,
        markets: Vec<&str>,
    ) -> Result<Vec<NFTPurchaseResult>, String> {
        let mut results = Vec::new();

        for market in markets {
            if let Ok(result) = Self::list_nft_on_market(
                Arc::clone(&client),
                Arc::clone(&wallet),
                token_id,
                price,
                market,
            )
            .await
            {
                results.push(result);
            }
        }

        Ok(results)
    }

    /// List NFT on single marketplace
    pub async fn list_nft_on_market(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        token_id: &str,
        price: u64,
        market: &str,
    ) -> Result<NFTPurchaseResult, String> {
        let contract_call = Self::build_listing_call(token_id, price, market)?;
        match crate::contract::Contract::write(client, wallet, contract_call).await {
            Ok(result) => Ok(NFTPurchaseResult {
                success: true,
                transaction_hash: result.transaction_hash.clone(),
                marketplace: market.to_string(),
                total_cost: 0,
                gas_used: result.gas_used_as_u64(),
            }),
            Err(e) => Err(e),
        }
    }

    /// Build listing call
    fn build_listing_call(
        token_id: &str,
        price: u64,
        market: &str,
    ) -> Result<ContractCall, String> {
        let (module_address, module_name, function_name, arguments) = match market {
            "Topaz" => (
                TOPAZ.to_string(),
                "marketplace".to_string(),
                "list".to_string(),
                vec![json!(token_id), json!(price.to_string())],
            ),
            "Souffl3" => (
                SOUFFL3.to_string(),
                "market".to_string(),
                "list".to_string(),
                vec![json!(token_id), json!(price.to_string())],
            ),
            "BlueMove" => (
                BLUEMOVE.to_string(),
                "marketplace".to_string(),
                "list_token".to_string(),
                vec![json!(token_id), json!(price.to_string())],
            ),
            "Tradeport" => (
                TRADEPORT.to_string(),
                "marketplace".to_string(),
                "list_token".to_string(),
                vec![json!(token_id), json!(price.to_string())],
            ),
            "Wapal" => (
                WAPAL.to_string(),
                "market".to_string(),
                "list_nft".to_string(),
                vec![json!(token_id), json!(price.to_string())],
            ),
            _ => return Err("Unsupported marketplace for listing".to_string()),
        };
        Ok(ContractCall {
            module_address,
            module_name,
            function_name,
            type_arguments: vec![],
            arguments,
        })
    }

    /// Get marketplace statistics
    pub async fn get_market_stats(
        client: Arc<AptosClient>,
        collection: &str,
    ) -> Result<HashMap<String, MarketStats>, String> {
        let mut stats = HashMap::new();
        for market in Marketplaces::all_markets() {
            if let Ok(market_stats) =
                Self::get_single_market_stats(Arc::clone(&client), market, collection).await
            {
                stats.insert(market.to_string(), market_stats);
            }
        }
        Ok(stats)
    }

    async fn get_single_market_stats(
        client: Arc<AptosClient>,
        market_address: &str,
        collection: &str,
    ) -> Result<MarketStats, String> {
        todo!();
        Ok(MarketStats {
            volume_24h: 0,
            transactions_24h: 0,
            floor_price: 0,
            listed_count: 0,
        })
    }
}

/// Marketplace statistics
#[derive(Debug, Clone)]
pub struct MarketStats {
    pub volume_24h: u64,
    pub transactions_24h: u64,
    pub floor_price: u64,
    pub listed_count: u64,
}

/// NFT marketplace utilities
pub struct NFTMarketUtils;

impl NFTMarketUtils {
    /// Verify if NFT is delisted from all marketplaces
    pub async fn verify_delisted(client: Arc<AptosClient>, token_id: &str) -> Result<bool, String> {
        let listings = NFTMarketplaceAggregator::search_nft_listings(client, token_id).await?;
        Ok(listings.is_empty())
    }

    /// Get NFT listing status across all marketplaces
    pub async fn get_listing_status(
        client: Arc<AptosClient>,
        token_id: &str,
    ) -> Result<HashMap<String, bool>, String> {
        let mut status = HashMap::new();
        let listings =
            NFTMarketplaceAggregator::search_nft_listings(Arc::clone(&client), token_id).await?;

        for market in Marketplaces::all_markets() {
            let is_listed = listings
                .iter()
                .any(|listing| listing.marketplace.contains(market));
            status.insert(market.to_string(), is_listed);
        }

        Ok(status)
    }

    /// Get cross-market floor price
    pub async fn get_cross_market_floor_price(
        client: Arc<AptosClient>,
        collection: &str,
    ) -> Result<u64, String> {
        let mut min_price = u64::MAX;
        for market in Marketplaces::all_markets() {
            if let Ok(stats) = NFTMarketplaceAggregator::get_single_market_stats(
                Arc::clone(&client),
                market,
                collection,
            )
            .await
            {
                if stats.floor_price > 0 && stats.floor_price < min_price {
                    min_price = stats.floor_price;
                }
            }
        }
        if min_price == u64::MAX {
            Ok(0)
        } else {
            Ok(min_price)
        }
    }
}
