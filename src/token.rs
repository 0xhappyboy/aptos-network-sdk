/// This module is used for token management utilities to create, register, and manage tokens on Aptos.
use crate::{
    AptosClient,
    dex::DexAggregator,
    global::mainnet::{
        protocol_address::{
            ANIMESWAP_PROTOCOL_ADDRESS, AUXSWAP_PROTOCOL_ADDRESS, CELLANASWAP_PROTOCOL_ADDRESS,
            LIQUIDSWAP_PROTOCOL_ADDRESS, PANCAKESWAP_PROTOCOL_ADDRESS, THALA_PROTOCOL_ADDRESS,
        },
        sys_address::X_3,
        token_address::{USDC, USDT, WORMHOLE_USDC},
    },
};
use crate::{
    global::mainnet::{
        sys_address::X_1,
        sys_module::{coin, managed_coin},
    },
    types::ContractCall,
    wallet::Wallet,
};
use serde_json::Value;
use serde_json::json;
use std::sync::Arc;

/// token search manager
pub struct TokenManager;

impl TokenManager {
    /// create token
    ///
    /// # Params
    /// client - aptos client
    /// wallet - wallet
    /// name - full name of the token
    /// symbol - token symbol
    /// decimals - number of decimal places
    /// initial_supply - Initial token supply
    ///
    /// # Example
    /// ```rust
    /// use std::sync::Arc;
    /// use crate::{AptosClient, Wallet, token::TokenManager};
    /// use crate::global::rpc::APTOS_MAINNET_URL;
    ///
    /// async fn example() -> Result<(), String> {
    /// let client = Arc::new(AptosClient::new(APTOS_MAINNET_URL));
    /// let wallet = Arc::new(Wallet::from_private_key("0x..."));
    ///
    /// let result = TokenManager::create_token(
    ///     client,
    ///     wallet,
    ///     "Test Token",
    ///     "TT",
    ///     8,
    ///     1_000_000_000,
    /// ).await?;
    ///  Ok(())
    /// }
    /// ```
    pub async fn create_token(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        name: &str,
        symbol: &str,
        decimals: u8,
        initial_supply: u64,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: X_1.to_string(),
            module_name: managed_coin::name.to_string(),
            function_name: managed_coin::initialize.to_string(),
            type_arguments: vec![],
            arguments: vec![
                json!(name),
                json!(symbol),
                json!(decimals),
                json!(initial_supply.to_string()),
            ],
        };
        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// register token
    ///
    /// # Params
    /// client - Aptos client
    /// wallet - Wallet
    /// token_type - Full token type string ("0x1::coin::CoinStore<0x123::my_token::MyToken>")
    ///
    /// # Example
    /// ```rust
    /// use std::sync::Arc;
    /// use crate::{AptosClient, Wallet, token::TokenManager};
    /// use crate::global::rpc::APTOS_MAINNET_URL;
    /// async fn example() -> Result<(), String> {
    /// let client = Arc::new(AptosClient::new(APTOS_MAINNET_URL));
    /// let wallet = Arc::new(Wallet::from_private_key("0x..."));
    /// let token_type = "0x123::my_token::MyToken";
    ///
    /// let result = TokenManager::register_token(client, wallet, token_type).await?;
    /// Ok(())
    /// }
    /// ```
    pub async fn register_token(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        token_type: &str,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: X_1.to_string(),
            module_name: coin::name.to_string(),
            function_name: coin::register.to_string(),
            type_arguments: vec![token_type.to_string()],
            arguments: vec![],
        };
        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// mint token
    ///
    /// # Arguments
    /// client - aptos client
    /// wallet - wallet
    /// token_type - full token type string
    /// recipient - recipient address
    /// amount - amount of tokens to mint
    ///
    /// # Example
    /// ```rust
    /// use std::sync::Arc;
    /// use crate::{AptosClient, Wallet, token::TokenManager};
    /// use crate::global::rpc::APTOS_MAINNET_URL;
    ///
    /// async fn example() -> Result<(), String> {
    /// let client = Arc::new(AptosClient::new(APTOS_MAINNET_URL));
    /// let wallet = Arc::new(Wallet::from_private_key("0x..."));
    /// let token_type = "0x123::my_token::MyToken";
    ///
    /// let result = TokenManager::mint_token(
    ///     client,
    ///     wallet,
    ///     token_type,
    ///     "0x789...",
    ///     100_000_000,
    /// ).await?;
    /// Ok(())
    /// }
    /// ```
    pub async fn mint_token(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        token_type: &str,
        recipient: &str,
        amount: u64,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: X_1.to_string(),
            module_name: managed_coin::name.to_string(),
            function_name: managed_coin::mint.to_string(),
            type_arguments: vec![token_type.to_string()],
            arguments: vec![json!(recipient), json!(amount.to_string())],
        };
        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// burn token
    pub async fn burn_token(
        client: Arc<AptosClient>,
        wallet: Arc<Wallet>,
        token_type: &str,
        amount: u64,
    ) -> Result<Value, String> {
        let contract_call = ContractCall {
            module_address: X_1.to_string(),
            module_name: managed_coin::name.to_string(),
            function_name: managed_coin::burn.to_string(),
            type_arguments: vec![token_type.to_string()],
            arguments: vec![json!(amount.to_string())],
        };
        crate::contract::Contract::write(client, wallet, contract_call)
            .await
            .map(|result| json!(result))
    }

    /// get token metadata
    ///
    /// # Params
    /// client - aptos client
    /// token_type - full token type string
    ///
    /// # Example
    /// ```rust
    /// use std::sync::Arc;
    /// use crate::{AptosClient, token::TokenManager};
    /// use crate::global::rpc::APTOS_MAINNET_URL;
    ///
    /// # async fn example() -> Result<(), String> {
    /// let client = Arc::new(AptosClient::new(APTOS_MAINNET_URL));
    /// let token_type = "0x1::aptos_coin::AptosCoin";
    ///
    /// let metadata = TokenManager::get_token_metadata(client, token_type).await?;
    /// println!("Token metadata: {:?}", metadata);
    /// Ok(())
    /// }
    /// ```
    pub async fn get_token_metadata(
        client: Arc<AptosClient>,
        token_type: &str,
    ) -> Result<Value, String> {
        let resource_type = format!("0x1::coin::CoinInfo<{}>", token_type);
        client
            .get_account_resource(X_1, &resource_type)
            .await
            .map(|opt| opt.map(|r| r.data).unwrap_or(Value::Null))
            .map_err(|e| e.to_string())
    }

    /// get token balance
    ///
    /// # Arguments
    /// client - aptos client instance
    /// address - account address to check balance for
    /// token_type - full token type string
    ///
    /// # Example
    /// ```rust
    /// use std::sync::Arc;
    /// use crate::{AptosClient, token::TokenManager};
    /// use crate::global::rpc::APTOS_MAINNET_URL;
    ///
    /// async fn example() -> Result<(), String> {
    /// let client = Arc::new(AptosClient::new(APTOS_MAINNET_URL));
    /// let address = "0x123...";
    /// let token_type = "0x1::aptos_coin::AptosCoin";
    ///
    /// let balance = TokenManager::get_token_balance(client, address, token_type).await?;
    /// println!("Balance: {}", balance);
    /// Ok(())
    /// }
    /// ```
    pub async fn get_token_balance(
        client: Arc<AptosClient>,
        address: &str,
        token_type: &str,
    ) -> Result<u64, String> {
        client.get_token_balance(address, token_type).await
    }
}

/// token utils
pub struct TokenUtils;

impl TokenUtils {
    /// Building standard token types
    ///
    /// # Params
    /// creator - token creator address
    /// collection - collection name
    /// name - token name
    ///
    /// # Example
    /// ```rust
    /// use crate::token::TokenUtils;
    /// use crate::global::rpc::APTOS_MAINNET_URL;
    ///
    /// let token_type = TokenUtils::build_standard_token_type(
    ///     "0x123",
    ///     "my_collection",
    ///     "MyToken"
    /// );
    /// assert_eq!(token_type, "0x123::my_collection::MyToken");
    /// ```
    pub fn build_standard_token_type(creator: &str, collection: &str, name: &str) -> String {
        format!("{}::{}::{}", creator, collection, name)
    }

    /// parse token type
    ///
    /// # Arguments
    /// token_type - Full token type string
    ///
    /// # Example
    /// ```rust
    /// use crate::token::TokenUtils;
    ///
    /// let token_type = "0x123::my_collection::MyToken";
    /// if let Some((creator, collection, name)) = TokenUtils::parse_token_type(token_type) {
    ///     println!("Creator: {}, Collection: {}, Name: {}", creator, collection, name);
    /// }
    /// ```
    pub fn parse_token_type(token_type: &str) -> Option<(String, String, String)> {
        let parts: Vec<&str> = token_type.split("::").collect();
        if parts.len() == 3 {
            Some((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
            ))
        } else {
            None
        }
    }
    /// verify token address format
    ///
    /// # Params
    /// address - Address string to validate
    ///
    /// # Example
    /// ```rust
    /// use crate::token::TokenUtils;
    ///
    /// assert!(TokenUtils::is_valid_token_address("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"));
    /// assert!(!TokenUtils::is_valid_token_address("invalid_address"));
    /// ```
    pub fn is_valid_token_address(address: &str) -> bool {
        address.starts_with("0x") && address.len() == 66
    }
}

pub struct TokenSearchManager;

impl TokenSearchManager {
    /// get token by symbol
    ///
    /// # Params
    /// client - aptos client
    /// symbol - token symbol
    ///
    /// # Example
    /// ```rust
    /// use std::sync::Arc;
    /// use crate::{AptosClient, token::TokenSearchManager};
    /// use crate::global::rpc::APTOS_MAINNET_URL;
    ///
    /// async fn example() -> Result<(), String> {
    /// let client = Arc::new(AptosClient::new(APTOS_MAINNET_URL));
    ///
    /// let results = TokenSearchManager::get_token_by_symbol(client, "USDC").await?;
    /// for token in results {
    ///     println!("Found token: {} ({})", token.symbol, token.address);
    /// }
    /// Ok(())
    /// }
    pub async fn get_token_by_symbol(
        client: Arc<AptosClient>,
        symbol: &str,
    ) -> Result<Vec<TokenSearchResult>, String> {
        let mut results = Vec::new();
        let search_symbol = symbol.to_uppercase();
        let protocol_addresses = vec![
            // Thala
            THALA_PROTOCOL_ADDRESS,
            // Liquidswap
            LIQUIDSWAP_PROTOCOL_ADDRESS,
            // PancakeSwap
            PANCAKESWAP_PROTOCOL_ADDRESS,
            USDC,          // USDC
            USDT,          // USDT
            WORMHOLE_USDC, // Wormhole USDC
        ];
        for address in protocol_addresses {
            if let Ok(modules) = client.get_account_module_vec(address).await {
                for module in modules {
                    if let Some(abi) = module.abi {
                        if let Some(abi_obj) = abi.as_object() {
                            if let Some(token_info) =
                                Self::get_token_info_from_abi(abi_obj, address)
                            {
                                if token_info.symbol.to_uppercase().contains(&search_symbol) {
                                    results.push(token_info);
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Ok(coin_infos) =
            Self::get_coin_infos_by_symbol(Arc::clone(&client), &search_symbol).await
        {
            results.extend(coin_infos);
        }
        if let Ok(pool_tokens) =
            Self::search_tokens_from_pools(Arc::clone(&client), &search_symbol).await
        {
            for token in pool_tokens {
                if !results.iter().any(|r| r.address == token.address) {
                    results.push(token);
                }
            }
        }
        results.sort_by(|a, b| a.symbol.cmp(&b.symbol));
        results.dedup_by(|a, b| a.address == b.address);
        Ok(results)
    }

    /// get_token_info_from_abi
    fn get_token_info_from_abi(
        abi: &serde_json::Map<String, Value>,
        module_address: &str,
    ) -> Option<TokenSearchResult> {
        if let Some(structs) = abi.get("structs").and_then(|v| v.as_array()) {
            for struct_info in structs {
                if let Some(name) = struct_info.get("name").and_then(|v| v.as_str()) {
                    if name.contains("CoinInfo") || name.contains("Token") {
                        if let (Some(symbol), Some(name_val), Some(decimals)) = (
                            Self::get_string_field(struct_info, "symbol"),
                            Self::get_string_field(struct_info, "name"),
                            Self::get_u64_field(struct_info, "decimals"),
                        ) {
                            let module_name = abi
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            let address =
                                format!("{}::{}::{}", module_address, module_name, symbol);
                            return Some(TokenSearchResult {
                                symbol: symbol.to_string(),
                                address,
                                name: name_val.to_string(),
                                decimals: decimals as u8,
                                verified: Self::is_verified_token(&symbol),
                            });
                        }
                    }
                }
            }
        }
        None
    }

    /// get string field from struct info
    fn get_string_field(struct_info: &Value, field_name: &str) -> Option<String> {
        if let Some(fields) = struct_info.get("fields").and_then(|v| v.as_array()) {
            for field in fields {
                if let (Some(name), Some(value)) = (
                    field.get("name").and_then(|v| v.as_str()),
                    field.get("value").and_then(|v| v.as_str()),
                ) {
                    if name == field_name {
                        return Some(value.to_string());
                    }
                }
            }
        }
        None
    }

    /// get u64 field
    fn get_u64_field(struct_info: &Value, field_name: &str) -> Option<u64> {
        if let Some(fields) = struct_info.get("fields").and_then(|v| v.as_array()) {
            for field in fields {
                if let (Some(name), Some(value)) = (
                    field.get("name").and_then(|v| v.as_str()),
                    field.get("value").and_then(|v| v.as_u64()),
                ) {
                    if name == field_name {
                        return Some(value);
                    }
                }
            }
        }
        None
    }

    /// get coin infos by symbol
    async fn get_coin_infos_by_symbol(
        client: Arc<AptosClient>,
        symbol: &str,
    ) -> Result<Vec<TokenSearchResult>, String> {
        let mut results = Vec::new();
        let known_accounts = vec![X_1, X_3];
        for account in known_accounts {
            if let Ok(resources) = client.get_account_resource_vec(account).await {
                for resource in resources {
                    if resource.r#type.starts_with("0x1::coin::CoinInfo<") {
                        if let Some(token_info) =
                            Self::get_token_info_from_resource(&resource, symbol).await
                        {
                            results.push(token_info);
                        }
                    }
                }
            }
        }
        Ok(results)
    }

    /// get token info from resource
    async fn get_token_info_from_resource(
        resource: &crate::types::Resource,
        search_symbol: &str,
    ) -> Option<TokenSearchResult> {
        if let Value::Object(data) = &resource.data {
            if let (Some(symbol_value), Some(name_value), Some(decimals_value)) =
                (data.get("symbol"), data.get("name"), data.get("decimals"))
            {
                let symbol = symbol_value.as_str().unwrap_or("").to_string();
                let name = name_value.as_str().unwrap_or("").to_string();
                let decimals = decimals_value.as_u64().unwrap_or(0) as u8;
                if symbol
                    .to_uppercase()
                    .contains(&search_symbol.to_uppercase())
                {
                    let token_address = resource
                        .r#type
                        .trim_start_matches("0x1::coin::CoinInfo<")
                        .trim_end_matches('>')
                        .to_string();
                    let symbol_clone = symbol.clone();
                    return Some(TokenSearchResult {
                        symbol: symbol_clone,
                        address: token_address,
                        name,
                        decimals,
                        verified: Self::is_verified_token(&symbol),
                    });
                }
            }
        }
        None
    }

    /// search tokens from pools
    async fn search_tokens_from_pools(
        client: Arc<AptosClient>,
        search_symbol: &str,
    ) -> Result<Vec<TokenSearchResult>, String> {
        let mut results = Vec::new();
        // Check the liquidity pools of major DEXs
        let dex_addresses = vec![
            LIQUIDSWAP_PROTOCOL_ADDRESS,
            THALA_PROTOCOL_ADDRESS,
            PANCAKESWAP_PROTOCOL_ADDRESS,
            ANIMESWAP_PROTOCOL_ADDRESS,
            AUXSWAP_PROTOCOL_ADDRESS,
            CELLANASWAP_PROTOCOL_ADDRESS,
        ];
        for dex_address in dex_addresses {
            if let Ok(resources) = client.get_account_resource_vec(dex_address).await {
                for resource in resources {
                    if resource.r#type.contains("::liquidity_pool::")
                        || resource.r#type.contains("::Pool<")
                        || resource.r#type.contains("::LiquidityPool<")
                    {
                        let token_types = Self::get_token_types_from_pool(&resource.r#type);
                        for token_type in token_types {
                            if let Some(token_info) = Self::get_token_info_from_type(
                                Arc::clone(&client),
                                &token_type,
                                search_symbol,
                            )
                            .await
                            {
                                results.push(token_info);
                            }
                        }
                    }
                }
            }
        }
        Ok(results)
    }

    /// get token types from pool
    fn get_token_types_from_pool(pool_type: &str) -> Vec<String> {
        let mut token_types = Vec::new();
        if let Some(start_idx) = pool_type.find('<') {
            if let Some(end_idx) = pool_type.rfind('>') {
                let type_args = &pool_type[start_idx + 1..end_idx];
                let types: Vec<&str> = type_args.split(',').map(|s| s.trim()).collect();
                for token_type in types {
                    token_types.push(token_type.to_string());
                }
            }
        }
        token_types
    }

    /// get token info from type
    async fn get_token_info_from_type(
        client: Arc<AptosClient>,
        token_type: &str,
        search_symbol: &str,
    ) -> Option<TokenSearchResult> {
        let parts: Vec<&str> = token_type.split("::").collect();
        if parts.len() >= 3 {
            let address = parts[0];
            let module = parts[1];
            let token_name = parts[2];
            if token_name
                .to_uppercase()
                .contains(&search_symbol.to_uppercase())
            {
                if let Ok(metadata) =
                    DexAggregator::get_token_metadata(Arc::clone(&client), token_type).await
                {
                    return Some(TokenSearchResult {
                        symbol: token_name.to_string(),
                        address: token_type.to_string(),
                        name: metadata.name,
                        decimals: metadata.decimals,
                        verified: Self::is_verified_token(token_name),
                    });
                }
                return Some(TokenSearchResult {
                    symbol: token_name.to_string(),
                    address: token_type.to_string(),
                    name: format!("{} Token", token_name),
                    decimals: 8, 
                    verified: Self::is_verified_token(token_name),
                });
            }
        }
        None
    }

    /// is verified token
    fn is_verified_token(symbol: &str) -> bool {
        let verified_tokens = vec!["APT", "USDC", "USDT", "THL", "CAKE", "CELL"];
        verified_tokens.contains(&symbol.to_uppercase().as_str())
    }

    /// get top token vec
    ///
    /// # Params
    /// client - aptos client
    ///
    /// # Example
    /// ```rust
    /// use std::sync::Arc;
    /// use crate::{AptosClient, token::TokenSearchManager};
    /// use crate::global::rpc::APTOS_MAINNET_URL;
    ///
    /// async fn example() -> Result<(), String> {
    /// let client = Arc::new(AptosClient::new(APTOS_MAINNET_URL));
    ///
    /// let top_tokens = TokenSearchManager::get_top_token_vec(client).await?;
    /// for token in top_tokens {
    ///     println!("{}: ${} (24h volume: {})", token.symbol, token.price, token.volume_24h);
    /// }
    /// Ok(())
    /// }
    /// ```
    pub async fn get_top_token_vec(client: Arc<AptosClient>) -> Result<Vec<TopToken>, String> {
        let mut top_tokens = Vec::new();
        let base_token = "0x1::aptos_coin::AptosCoin";
        if let Ok(resources) = client
            .get_account_resource_vec(LIQUIDSWAP_PROTOCOL_ADDRESS)
            .await
        {
            for resource in resources {
                if resource.r#type.contains("::liquidity_pool::LiquidityPool<")
                    && resource.r#type.contains(base_token)
                {
                    if let Some(token_info) =
                        Self::get_token_from_pool_resource(&resource, base_token).await
                    {
                        let price =
                            Self::estimate_token_price(Arc::clone(&client), &token_info.address)
                                .await
                                .unwrap_or(0.0);
                        let volume =
                            Self::estimate_volume(Arc::clone(&client), &token_info.address)
                                .await
                                .unwrap_or(0);
                        top_tokens.push(TopToken {
                            symbol: token_info.symbol,
                            address: token_info.address,
                            name: token_info.name,
                            price,
                            volume_24h: volume,
                            change_24h: 0.0,
                        });
                    }
                }
            }
        }
        // sort by transaction volume
        top_tokens.sort_by(|a, b| b.volume_24h.cmp(&a.volume_24h));
        // limit 10
        if top_tokens.len() > 10 {
            top_tokens.truncate(10);
        }
        Ok(top_tokens)
    }

    /// get token from pool resource
    async fn get_token_from_pool_resource(
        resource: &crate::types::Resource,
        base_token: &str,
    ) -> Option<TokenSearchResult> {
        let token_types = Self::get_token_types_from_pool(&resource.r#type);
        for token_type in token_types {
            if token_type != base_token {
                let parts: Vec<&str> = token_type.split("::").collect();
                if parts.len() >= 3 {
                    let symbol = parts[2].to_string();
                    return Some(TokenSearchResult {
                        symbol: symbol.clone(),
                        address: token_type,
                        name: format!("{} Token", symbol),
                        decimals: 8,
                        verified: Self::is_verified_token(&symbol),
                    });
                }
            }
        }
        None
    }

    /// estimate token price
    async fn estimate_token_price(
        client: Arc<AptosClient>,
        token_address: &str,
    ) -> Result<f64, String> {
        let base_token = "0x1::aptos_coin::AptosCoin";
        DexAggregator::get_token_price(client, token_address)
            .await
            .map(|prices| prices.first().map(|p| p.price).unwrap_or(0.0))
    }

    /// estimate volume
    async fn estimate_volume(client: Arc<AptosClient>, token_address: &str) -> Result<u64, String> {
        let volume = match token_address {
            "0x1::aptos_coin::AptosCoin" => 5_000_000_000, // apt
            addr if addr.contains("usd") || addr.contains("stable") => 2_000_000_000,
            addr if addr.contains("wormhole") => 1_000_000_000,
            _ => 500_000_000,
        };
        Ok(volume)
    }

    /// get token trading pairs
    ///
    /// # Params
    /// client - aptos client
    /// token_address - token address
    ///
    /// # Example
    /// ```rust
    /// use std::sync::Arc;
    /// use crate::{AptosClient, token::TokenSearchManager};
    /// use crate::global::rpc::APTOS_MAINNET_URL;
    ///
    /// async fn example() -> Result<(), String> {
    /// let client = Arc::new(AptosClient::new(APTOS_MAINNET_URL));
    /// let token_address = "0x1::aptos_coin::AptosCoin";
    ///
    /// let pairs = TokenSearchManager::get_token_trading_pairs(client, token_address).await?;
    /// for pair in pairs {
    ///     println!("{} - {} on {:?}", pair.token_a, pair.token_b, pair.dexes);
    /// }
    /// Ok(())
    /// }
    /// ```
    pub async fn get_token_trading_pairs(
        client: Arc<AptosClient>,
        token_address: &str,
    ) -> Result<Vec<TradePair>, String> {
        DexAggregator::find_token_liquidity_pools(client, token_address)
            .await
            .map(|pools| {
                pools
                    .into_iter()
                    .map(|pool| TradePair {
                        token_a: pool.token_a,
                        token_b: pool.token_b,
                        dexes: vec![pool.dex],
                        total_liquidity: pool.liquidity,
                    })
                    .collect()
            })
    }
}

/// token search result
#[derive(Debug, Clone)]
pub struct TokenSearchResult {
    pub symbol: String,
    pub address: String,
    pub name: String,
    pub decimals: u8,
    pub verified: bool,
}

/// top token
#[derive(Debug, Clone)]
pub struct TopToken {
    pub symbol: String,
    pub address: String,
    pub name: String,
    pub price: f64,
    pub volume_24h: u64,
    pub change_24h: f64,
}

/// new token
#[derive(Debug, Clone)]
pub struct NewToken {
    pub symbol: String,
    pub address: String,
    pub name: String,
    pub launch_time: u64,
    pub initial_liquidity: u64,
}

/// trade pair
#[derive(Debug, Clone)]
pub struct TradePair {
    pub token_a: String,
    pub token_b: String,
    pub dexes: Vec<String>,
    pub total_liquidity: u64,
}
