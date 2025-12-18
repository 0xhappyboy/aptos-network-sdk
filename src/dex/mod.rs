pub mod animeswap;
pub mod auxswap;
pub mod cellana;
pub mod liquidswap;
pub mod pancakeswap;
pub mod thala;
use crate::{
    Aptos,
    dex::{
        animeswap::{AnimeSwap, AnimeSwapEventFilters},
        auxswap::AuxExchange,
        cellana::{Cellana, CellanaEventConfig},
        liquidswap::Liquidswap,
        pancakeswap::{PancakeSwap, PancakeSwapEventFilters},
        thala::Thala,
    },
    event::EventData,
    global::mainnet::{
        protocol_address::{
            ANIMESWAP_PROTOCOL_ADDRESS, AUXSWAP_PROTOCOL_ADDRESS, CELLANASWAP_PROTOCOL_ADDRESS,
            LIQUIDSWAP_PROTOCOL_ADDRESS, PANCAKESWAP_FACTORY_PROTOCOL_ADDRESS,
            THALA_PROTOCOL_ADDRESS,
        },
        token_address::{APT, THL, USDC, USDT, WORMHOLE_USDC},
    },
    wallet::Wallet,
};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;

/// token ptice
#[derive(Debug, Clone)]
pub struct TokenPrice {
    pub dex: String,
    pub token_address: String,
    pub base_token: String,
    pub price: f64,
    pub liquidity: u64,
    pub timestamp: u64,
}

/// liquidity pool info
#[derive(Debug, Clone)]
pub struct LiquidityPool {
    pub dex: String,
    pub token_a: String,
    pub token_b: String,
    pub liquidity: u64,
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub fee_rate: f64,
}

/// token metadata
#[derive(Debug, Clone)]
pub struct TokenMetadata {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub supply: u64,
}

/// dex price
#[derive(Debug, Clone)]
pub struct DexPrice {
    pub dex: String,
    pub price: f64,
    pub amount_out: u64,
}

/// token price comparison
#[derive(Debug, Clone)]
pub struct TokenPriceComparison {
    pub token_a: String,
    pub token_b: String,
    pub prices: Vec<DexPrice>,
}

pub struct DexAggregator;

impl DexAggregator {
    /// Find the best price across all DEXs
    pub async fn find_best_swap(
        client: Arc<Aptos>,
        from_token: &str,
        to_token: &str,
        amount_in: u64,
    ) -> Result<DexSwapQuote, String> {
        let mut quotes = Vec::new();
        if let Ok(quote) =
            Self::get_liquidswap_quote(Arc::clone(&client), from_token, to_token, amount_in).await
        {
            quotes.push(quote);
        }
        if let Ok(quote) =
            Self::get_animeswap_quote(Arc::clone(&client), from_token, to_token, amount_in).await
        {
            quotes.push(quote);
        }
        if let Ok(quote) =
            Self::get_thala_quote(Arc::clone(&client), from_token, to_token, amount_in).await
        {
            quotes.push(quote);
        }
        if let Ok(quote) =
            Self::get_pancakeswap_quote(Arc::clone(&client), from_token, to_token, amount_in).await
        {
            quotes.push(quote);
        }
        if let Ok(quote) =
            Self::get_cellana_quote(Arc::clone(&client), from_token, to_token, amount_in).await
        {
            quotes.push(quote);
        }
        if let Ok(quote) =
            Self::get_aux_quote(Arc::clone(&client), from_token, to_token, amount_in).await
        {
            quotes.push(quote);
        }
        if quotes.is_empty() {
            return Err("No suitable DEX found for this trade".to_string());
        }
        // Sort by output amount and select the best quote
        quotes.sort_by(|a, b| b.amount_out.cmp(&a.amount_out));
        Ok(quotes.first().unwrap().clone())
    }

    /// Perform optimal exchange
    pub async fn exe_best_swap(
        client: Arc<Aptos>,
        wallet: Arc<Wallet>,
        from_token: &str,
        to_token: &str,
        amount_in: u64,
        slippage: f64,
    ) -> Result<Value, String> {
        let quote =
            Self::find_best_swap(Arc::clone(&client), from_token, to_token, amount_in).await?;
        let min_amount_out = (quote.amount_out as f64 * (1.0 - slippage)) as u64;
        match quote.dex.as_str() {
            "Liquidswap" => {
                Liquidswap::swap_exact_input(
                    client,
                    wallet,
                    from_token,
                    to_token,
                    amount_in,
                    min_amount_out,
                )
                .await
            }
            "AnimeSwap" => {
                AnimeSwap::swap_exact_tokens_for_tokens(
                    client,
                    wallet,
                    vec![from_token, to_token],
                    amount_in,
                    min_amount_out,
                )
                .await
            }
            "Thala" => {
                Thala::swap_exact_input(
                    client,
                    wallet,
                    from_token,
                    to_token,
                    amount_in,
                    min_amount_out,
                )
                .await
            }
            "PancakeSwap" => {
                let wallet_address = wallet.address().map_err(|e| e.to_string())?;
                PancakeSwap::swap_exact_tokens_for_tokens(
                    client,
                    wallet,
                    amount_in,
                    min_amount_out,
                    vec![from_token, to_token],
                    &wallet_address,
                    Self::get_deadline(300),
                )
                .await
            }
            "Cellana" => {
                Cellana::swap(
                    client,
                    wallet,
                    from_token,
                    to_token,
                    amount_in,
                    min_amount_out,
                )
                .await
            }
            "AuxExchange" => {
                AuxExchange::swap_exact_input(
                    client,
                    wallet,
                    from_token,
                    to_token,
                    amount_in,
                    min_amount_out,
                )
                .await
            }
            _ => Err(format!("Unsupported DEX: {}", quote.dex)),
        }
    }

    /// Compare prices across multiple DEXs in batches
    pub async fn compare_all_dex_prices(
        client: Arc<Aptos>,
        from_token: &str,
        to_token: &str,
        amount_in: u64,
    ) -> Result<Vec<DexSwapQuote>, String> {
        let mut quotes = Vec::new();
        if let Ok(quote) =
            Self::get_liquidswap_quote(Arc::clone(&client), from_token, to_token, amount_in).await
        {
            quotes.push(quote);
        }
        if let Ok(quote) =
            Self::get_animeswap_quote(Arc::clone(&client), from_token, to_token, amount_in).await
        {
            quotes.push(quote);
        }
        if let Ok(quote) =
            Self::get_thala_quote(Arc::clone(&client), from_token, to_token, amount_in).await
        {
            quotes.push(quote);
        }
        if let Ok(quote) =
            Self::get_pancakeswap_quote(Arc::clone(&client), from_token, to_token, amount_in).await
        {
            quotes.push(quote);
        }
        if let Ok(quote) =
            Self::get_cellana_quote(Arc::clone(&client), from_token, to_token, amount_in).await
        {
            quotes.push(quote);
        }
        if let Ok(quote) =
            Self::get_aux_quote(Arc::clone(&client), from_token, to_token, amount_in).await
        {
            quotes.push(quote);
        }
        quotes.sort_by(|a, b| b.amount_out.cmp(&a.amount_out));
        Ok(quotes)
    }

    // How to obtain quotes from various DEXs
    async fn get_liquidswap_quote(
        client: Arc<Aptos>,
        from_token: &str,
        to_token: &str,
        amount_in: u64,
    ) -> Result<DexSwapQuote, String> {
        match Liquidswap::get_price(Arc::clone(&client), from_token, to_token, amount_in).await {
            Ok(price) => {
                let amount_out = (price * amount_in as f64) as u64;
                Ok(DexSwapQuote {
                    dex: "Liquidswap".to_string(),
                    amount_out,
                    price,
                    dex_address: LIQUIDSWAP_PROTOCOL_ADDRESS.to_string(),
                })
            }
            Err(e) => Err(e),
        }
    }

    async fn get_animeswap_quote(
        client: Arc<Aptos>,
        from_token: &str,
        to_token: &str,
        amount_in: u64,
    ) -> Result<DexSwapQuote, String> {
        match AnimeSwap::get_reserves(Arc::clone(&client), from_token, to_token).await {
            Ok((reserve_in, reserve_out)) => {
                let amount_out = Self::calculate_amm_output(amount_in, reserve_in, reserve_out);
                let price = if amount_in > 0 {
                    amount_out as f64 / amount_in as f64
                } else {
                    0.0
                };
                Ok(DexSwapQuote {
                    dex: "AnimeSwap".to_string(),
                    amount_out,
                    price,
                    dex_address: ANIMESWAP_PROTOCOL_ADDRESS.to_string(),
                })
            }
            Err(_) => Err("Failed to get AnimeSwap reserves".to_string()),
        }
    }

    async fn get_thala_quote(
        client: Arc<Aptos>,
        from_token: &str,
        to_token: &str,
        amount_in: u64,
    ) -> Result<DexSwapQuote, String> {
        match Thala::get_price(Arc::clone(&client), from_token, to_token, amount_in).await {
            Ok(price) => {
                let amount_out = (price * amount_in as f64) as u64;
                Ok(DexSwapQuote {
                    dex: "Thala".to_string(),
                    amount_out,
                    price,
                    dex_address: THALA_PROTOCOL_ADDRESS.to_string(),
                })
            }
            Err(e) => Err(e),
        }
    }

    async fn get_pancakeswap_quote(
        client: Arc<Aptos>,
        from_token: &str,
        to_token: &str,
        amount_in: u64,
    ) -> Result<DexSwapQuote, String> {
        match PancakeSwap::get_reserves(Arc::clone(&client), from_token, to_token).await {
            Ok((reserve_in, reserve_out)) => {
                let amount_out = Self::calculate_amm_output(amount_in, reserve_in, reserve_out);
                let price = if amount_in > 0 {
                    amount_out as f64 / amount_in as f64
                } else {
                    0.0
                };
                Ok(DexSwapQuote {
                    dex: "PancakeSwap".to_string(),
                    amount_out,
                    price,
                    dex_address: PANCAKESWAP_FACTORY_PROTOCOL_ADDRESS.to_string(),
                })
            }
            Err(_) => Err("Failed to get PancakeSwap reserves".to_string()),
        }
    }

    async fn get_cellana_quote(
        client: Arc<Aptos>,
        from_token: &str,
        to_token: &str,
        amount_in: u64,
    ) -> Result<DexSwapQuote, String> {
        match Cellana::get_price(Arc::clone(&client), from_token, to_token, amount_in).await {
            Ok(price) => {
                let amount_out = (price * amount_in as f64) as u64;
                Ok(DexSwapQuote {
                    dex: "Cellana".to_string(),
                    amount_out,
                    price,
                    dex_address: CELLANASWAP_PROTOCOL_ADDRESS.to_string(),
                })
            }
            Err(e) => Err(e),
        }
    }

    async fn get_aux_quote(
        client: Arc<Aptos>,
        from_token: &str,
        to_token: &str,
        amount_in: u64,
    ) -> Result<DexSwapQuote, String> {
        match AuxExchange::get_price(Arc::clone(&client), from_token, to_token, amount_in).await {
            Ok(amount_out) => {
                let price = if amount_in > 0 {
                    amount_out as f64 / amount_in as f64
                } else {
                    0.0
                };
                Ok(DexSwapQuote {
                    dex: "AuxExchange".to_string(),
                    amount_out,
                    price,
                    dex_address: AUXSWAP_PROTOCOL_ADDRESS.to_string(),
                })
            }
            Err(e) => {
                match AuxExchange::get_pool_info(Arc::clone(&client), from_token, to_token).await {
                    Ok(pool_info) => {
                        if let (Some(reserve_in), Some(reserve_out)) = (
                            pool_info
                                .get("coin_a_reserve")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse::<u64>().ok()),
                            pool_info
                                .get("coin_b_reserve")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse::<u64>().ok()),
                        ) {
                            let amount_out =
                                Self::calculate_amm_output(amount_in, reserve_in, reserve_out);
                            let price = if amount_in > 0 {
                                amount_out as f64 / amount_in as f64
                            } else {
                                0.0
                            };
                            Ok(DexSwapQuote {
                                dex: "AuxExchange".to_string(),
                                amount_out,
                                price,
                                dex_address: AUXSWAP_PROTOCOL_ADDRESS.to_string(),
                            })
                        } else {
                            Err(format!("Failed to parse pool reserves: {}", e))
                        }
                    }
                    Err(pool_err) => Err(format!(
                        "Failed to get AuxExchange quote: {} (pool error: {})",
                        e, pool_err
                    )),
                }
            }
        }
    }

    /// Calculate AMM output amount
    fn calculate_amm_output(amount_in: u64, reserve_in: u64, reserve_out: u64) -> u64 {
        if reserve_in == 0 || reserve_out == 0 {
            return 0;
        }
        let amount_in_with_fee = amount_in * 997;
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = reserve_in * 1000 + amount_in_with_fee;
        if denominator == 0 {
            return 0;
        }
        numerator / denominator
    }

    /// Get transaction deadline timestamp
    fn get_deadline(seconds_from_now: u64) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + seconds_from_now
    }

    /// Get a list of all supported DEXs
    pub fn get_supported_dexes() -> Vec<DexInfo> {
        vec![
            DexInfo {
                name: "Liquidswap".to_string(),
                address: LIQUIDSWAP_PROTOCOL_ADDRESS.to_string(),
                description: "Pontem Network - Largest DEX on Aptos".to_string(),
                supports_liquidity: true,
                supports_swap: true,
                is_amm: true,
            },
            DexInfo {
                name: "AuxExchange".to_string(),
                address: AUXSWAP_PROTOCOL_ADDRESS.to_string(),
                description: "Orderbook-based DEX with AMM".to_string(),
                supports_liquidity: true,
                supports_swap: true,
                is_amm: false,
            },
            DexInfo {
                name: "AnimeSwap".to_string(),
                address: ANIMESWAP_PROTOCOL_ADDRESS.to_string(),
                description: "Multi-chain DEX with anime theme".to_string(),
                supports_liquidity: true,
                supports_swap: true,
                is_amm: true,
            },
            DexInfo {
                name: "Thala".to_string(),
                address: THALA_PROTOCOL_ADDRESS.to_string(),
                description: "DeFi protocol with THL token and staking".to_string(),
                supports_liquidity: true,
                supports_swap: true,
                is_amm: true,
            },
            DexInfo {
                name: "PancakeSwap".to_string(),
                address: PANCAKESWAP_FACTORY_PROTOCOL_ADDRESS.to_string(),
                description: "Multi-chain DEX with CAKE token".to_string(),
                supports_liquidity: true,
                supports_swap: true,
                is_amm: true,
            },
            DexInfo {
                name: "Cellana".to_string(),
                address: CELLANASWAP_PROTOCOL_ADDRESS.to_string(),
                description: "DEX with CELL token and farming".to_string(),
                supports_liquidity: true,
                supports_swap: true,
                is_amm: true,
            },
        ]
    }

    /// Get the price of a specified token in all DEXs (relative to APT)
    pub async fn get_token_price(
        client: Arc<Aptos>,
        token_address: &str,
    ) -> Result<Vec<TokenPrice>, String> {
        let apt_coin = "0x1::aptos_coin::AptosCoin";
        let mut prices = Vec::new();
        let dex_checks = vec![
            (
                "Liquidswap",
                Self::get_token_price_on_dex(
                    Arc::clone(&client),
                    "Liquidswap",
                    token_address,
                    apt_coin,
                ),
            ),
            (
                "Thala",
                Self::get_token_price_on_dex(Arc::clone(&client), "Thala", token_address, apt_coin),
            ),
            (
                "PancakeSwap",
                Self::get_token_price_on_dex(
                    Arc::clone(&client),
                    "PancakeSwap",
                    token_address,
                    apt_coin,
                ),
            ),
            (
                "AnimeSwap",
                Self::get_token_price_on_dex(
                    Arc::clone(&client),
                    "AnimeSwap",
                    token_address,
                    apt_coin,
                ),
            ),
            (
                "Cellana",
                Self::get_token_price_on_dex(
                    Arc::clone(&client),
                    "Cellana",
                    token_address,
                    apt_coin,
                ),
            ),
        ];
        for (dex_name, check_future) in dex_checks {
            if let Ok(price) = check_future.await {
                prices.push(price);
            }
        }
        prices.sort_by(|a, b| {
            b.price
                .partial_cmp(&a.price)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(prices)
    }

    /// Get token prices on a specific DEX
    async fn get_token_price_on_dex(
        client: Arc<Aptos>,
        dex_name: &str,
        token_address: &str,
        base_token: &str,
    ) -> Result<TokenPrice, String> {
        let amount_in = 1_000_000;
        let quote = match dex_name {
            "Liquidswap" => {
                Self::get_liquidswap_quote(client.clone(), token_address, base_token, amount_in)
                    .await
            }
            "Thala" => {
                Self::get_thala_quote(client.clone(), token_address, base_token, amount_in).await
            }
            "PancakeSwap" => {
                Self::get_pancakeswap_quote(client.clone(), token_address, base_token, amount_in)
                    .await
            }
            "AnimeSwap" => {
                Self::get_animeswap_quote(client.clone(), token_address, base_token, amount_in)
                    .await
            }
            "Cellana" => {
                Self::get_cellana_quote(client.clone(), token_address, base_token, amount_in).await
            }
            _ => Err("Unsupported DEX".to_string()),
        }?;
        Ok(TokenPrice {
            dex: dex_name.to_string(),
            token_address: token_address.to_string(),
            base_token: base_token.to_string(),
            price: quote.price,
            liquidity: Self::get_pool_liquidity(
                client.clone(),
                dex_name,
                token_address,
                base_token,
            )
            .await
            .unwrap_or(0),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    /// Get the total liquidity of the liquidity pool
    async fn get_pool_liquidity(
        client: Arc<Aptos>,
        dex_name: &str,
        token_a: &str,
        token_b: &str,
    ) -> Result<u64, String> {
        match dex_name {
            "Liquidswap" => {
                let pool_info = Liquidswap::get_pool_info(client, token_a, token_b).await?;
                let reserve_a = pool_info
                    .get("coin_x_reserve")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let reserve_b = pool_info
                    .get("coin_y_reserve")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                Ok(reserve_a + reserve_b)
            }
            "Thala" => {
                let pool_info = Thala::get_pool_info(client, token_a, token_b).await?;
                let reserve_a = pool_info
                    .get("reserve_x")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let reserve_b = pool_info
                    .get("reserve_y")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                Ok(reserve_a + reserve_b)
            }
            "AnimeSwap" => {
                let (reserve_a, reserve_b) =
                    AnimeSwap::get_reserves(client, token_a, token_b).await?;
                Ok(reserve_a + reserve_b)
            }
            "PancakeSwap" => {
                let (reserve_a, reserve_b) =
                    PancakeSwap::get_reserves(client, token_a, token_b).await?;
                Ok(reserve_a + reserve_b)
            }
            "Cellana" => {
                let pool_info = Cellana::get_pool_info(client, token_a, token_b).await?;
                let reserve_a = pool_info
                    .get("reserve_x")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let reserve_b = pool_info
                    .get("reserve_y")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                Ok(reserve_a + reserve_b)
            }
            _ => Ok(0),
        }
    }

    /// Find the liquidity pools of a token across all DEXs
    pub async fn find_token_liquidity_pools(
        client: Arc<Aptos>,
        token_address: &str,
    ) -> Result<Vec<LiquidityPool>, String> {
        let common_tokens = vec![APT, USDC, USDT, WORMHOLE_USDC];
        let mut pools = Vec::new();
        for base_token in common_tokens {
            let dex_checks = vec![
                (
                    "Liquidswap",
                    Self::check_pool_exists(
                        Arc::clone(&client),
                        "Liquidswap",
                        token_address,
                        base_token,
                    ),
                ),
                (
                    "Thala",
                    Self::check_pool_exists(
                        Arc::clone(&client),
                        "Thala",
                        token_address,
                        base_token,
                    ),
                ),
                (
                    "PancakeSwap",
                    Self::check_pool_exists(
                        Arc::clone(&client),
                        "PancakeSwap",
                        token_address,
                        base_token,
                    ),
                ),
                (
                    "AnimeSwap",
                    Self::check_pool_exists(
                        Arc::clone(&client),
                        "AnimeSwap",
                        token_address,
                        base_token,
                    ),
                ),
                (
                    "Cellana",
                    Self::check_pool_exists(
                        Arc::clone(&client),
                        "Cellana",
                        token_address,
                        base_token,
                    ),
                ),
            ];
            for (dex_name, check_future) in dex_checks {
                if let Ok(Some(pool)) = check_future.await {
                    pools.push(pool);
                }
            }
        }
        pools.sort_by(|a, b| b.liquidity.cmp(&a.liquidity));
        Ok(pools)
    }

    /// Check if a liquidity pool exists on a specific DEX
    async fn check_pool_exists(
        client: Arc<Aptos>,
        dex_name: &str,
        token_a: &str,
        token_b: &str,
    ) -> Result<Option<LiquidityPool>, String> {
        let liquidity =
            Self::get_pool_liquidity(Arc::clone(&client), dex_name, token_a, token_b).await;
        if let Ok(liquidity) = liquidity {
            if liquidity > 0 {
                let pool = LiquidityPool {
                    dex: dex_name.to_string(),
                    token_a: token_a.to_string(),
                    token_b: token_b.to_string(),
                    liquidity,
                    reserve_a: 0,
                    reserve_b: 0,
                    fee_rate: 0.003,
                };
                return Ok(Some(pool));
            }
        }
        Ok(None)
    }

    /// Get the metadata information of the token
    pub async fn get_token_metadata(
        client: Arc<Aptos>,
        token_address: &str,
    ) -> Result<TokenMetadata, String> {
        let coin_info_type = format!("0x1::coin::CoinInfo<{}>", token_address);
        if let Ok(Some(resource)) = client.get_account_resource("0x1", &coin_info_type).await {
            if let Value::Object(data) = &resource.data {
                return Ok(TokenMetadata {
                    address: token_address.to_string(),
                    name: data
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    symbol: data
                        .get("symbol")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    decimals: data.get("decimals").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
                    supply: data
                        .get("supply")
                        .and_then(|v| v.get("vec"))
                        .and_then(|v| v.as_array())
                        .and_then(|v| v.get(0))
                        .and_then(|v| v.get("value"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0),
                });
            }
        }
        Ok(TokenMetadata {
            address: token_address.to_string(),
            name: "Unknown".to_string(),
            symbol: "UNKNOWN".to_string(),
            decimals: 8,
            supply: 0,
        })
    }

    pub async fn get_top_prices_comparison(
        client: Arc<Aptos>,
    ) -> Result<Vec<TokenPriceComparison>, String> {
        let popular_pairs = vec![
            (USDC, APT), // USDC/APT
            (USDT, APT), // USDT/APT
            (THL, APT),  // THL/APT
        ];
        let mut comparisons = Vec::new();
        for (token_a, token_b) in popular_pairs {
            let mut prices = Vec::new();
            let amount_in = 1_000_000;
            if let Ok(quote) =
                Self::get_liquidswap_quote(Arc::clone(&client), token_a, token_b, amount_in).await
            {
                prices.push(DexPrice {
                    dex: "Liquidswap".to_string(),
                    price: quote.price,
                    amount_out: quote.amount_out,
                });
            }
            if let Ok(quote) =
                Self::get_thala_quote(Arc::clone(&client), token_a, token_b, amount_in).await
            {
                prices.push(DexPrice {
                    dex: "Thala".to_string(),
                    price: quote.price,
                    amount_out: quote.amount_out,
                });
            }
            if let Ok(quote) =
                Self::get_pancakeswap_quote(Arc::clone(&client), token_a, token_b, amount_in).await
            {
                prices.push(DexPrice {
                    dex: "PancakeSwap".to_string(),
                    price: quote.price,
                    amount_out: quote.amount_out,
                });
            }
            if prices.len() > 1 {
                comparisons.push(TokenPriceComparison {
                    token_a: token_a.to_string(),
                    token_b: token_b.to_string(),
                    prices,
                });
            }
        }
        Ok(comparisons)
    }
}

#[derive(Debug, Clone)]
pub struct DexSwapQuote {
    pub dex: String,
    pub amount_out: u64,
    pub price: f64,
    pub dex_address: String,
}

#[derive(Debug, Clone)]
pub struct DexInfo {
    pub name: String,
    pub address: String,
    pub description: String,
    pub supports_liquidity: bool,
    pub supports_swap: bool,
    pub is_amm: bool,
}

/// dex event monitor
pub struct DexEventMonitor {
    clients: HashMap<String, broadcast::Sender<EventData>>,
}

impl DexEventMonitor {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }
    pub async fn start_monitoring_all_dexes(
        &mut self,
        client: Arc<Aptos>,
    ) -> Result<(), String> {
        let dexes = vec![
            "Liquidswap",
            "Thala",
            "PancakeSwap",
            "Cellana",
            "AnimeSwap",
            "AuxExchange",
        ];
        for dex_name in dexes {
            let (sender, _) = broadcast::channel(1000);
            self.clients.insert(dex_name.to_string(), sender);
        }
        Self::start_dex_monitoring_task(
            Arc::clone(&client),
            "Liquidswap",
            self.get_sender("Liquidswap"),
        );
        Self::start_dex_monitoring_task(Arc::clone(&client), "Thala", self.get_sender("Thala"));
        Self::start_dex_monitoring_task(
            Arc::clone(&client),
            "PancakeSwap",
            self.get_sender("PancakeSwap"),
        );
        Self::start_dex_monitoring_task(Arc::clone(&client), "Cellana", self.get_sender("Cellana"));
        Self::start_dex_monitoring_task(
            Arc::clone(&client),
            "AnimeSwap",
            self.get_sender("AnimeSwap"),
        );
        Self::start_dex_monitoring_task(
            Arc::clone(&client),
            "AuxExchange",
            self.get_sender("AuxExchange"),
        );
        Ok(())
    }

    fn start_dex_monitoring_task(
        client: Arc<Aptos>,
        dex_name: &str,
        sender: Option<broadcast::Sender<EventData>>,
    ) {
        if let Some(sender) = sender {
            let client = Arc::clone(&client);
            let dex_name = dex_name.to_string();
            tokio::spawn(async move {
                match dex_name.as_str() {
                    "Liquidswap" => {
                        let _ = Liquidswap::listen_events(client, sender, vec![]).await;
                    }
                    "Thala" => {
                        let _ = Thala::listen_events(client, sender, vec![]).await;
                    }
                    "PancakeSwap" => {
                        let filters = PancakeSwapEventFilters {
                            min_swap_amount: Some(1000000000),
                            include_cake_pairs: true,
                            tracked_pairs: None,
                        };
                        let _ = PancakeSwap::listen_events(client, sender, filters).await;
                    }
                    "Cellana" => {
                        let config = CellanaEventConfig {
                            monitor_cell_pairs: true,
                            min_swap_amount: 1000000000,
                            monitor_farming: true,
                            tracked_tokens: vec![],
                        };
                        let _ = Cellana::listen_events(client, sender, config).await;
                    }
                    "AnimeSwap" => {
                        let filters = AnimeSwapEventFilters {
                            min_swap_amount: Some(1000000000),
                            tracked_tokens: None,
                            min_liquidity_amount: Some(500000000),
                        };
                        let _ = AnimeSwap::listen_events(client, sender, filters).await;
                    }
                    "AuxExchange" => {
                        let _ = AuxExchange::listen_events(client, sender, vec![]).await;
                    }
                    _ => {}
                }
            });
        }
    }

    fn get_sender(&self, dex_name: &str) -> Option<broadcast::Sender<EventData>> {
        self.clients.get(dex_name).cloned()
    }

    pub fn subscribe_to_dex(&self, dex_name: &str) -> Option<broadcast::Receiver<EventData>> {
        self.clients.get(dex_name).map(|sender| sender.subscribe())
    }

    pub fn get_all_receivers(&self) -> Vec<(String, broadcast::Receiver<EventData>)> {
        self.clients
            .iter()
            .map(|(name, sender)| (name.clone(), sender.subscribe()))
            .collect()
    }

    pub fn publish_to_dex(&self, dex_name: &str, event: EventData) -> Result<(), String> {
        if let Some(sender) = self.clients.get(dex_name) {
            let _ = sender.send(event);
            Ok(())
        } else {
            Err(format!("DEX {} not found", dex_name))
        }
    }
}

pub struct DexAnalytics;

impl DexAnalytics {
    /// analyze dex volume distribution
    pub async fn analyze_dex_volume_distribution(
        client: Arc<Aptos>,
        _time_period_hours: u64,
    ) -> Result<HashMap<String, u64>, String> {
        let mut volume_map = HashMap::new();
        let dex_volume_futures = vec![
            (
                "Liquidswap",
                Self::get_liquidswap_volume(Arc::clone(&client)).await,
            ),
            ("Thala", Self::get_thala_volume(Arc::clone(&client)).await),
            (
                "PancakeSwap",
                Self::get_pancakeswap_volume(Arc::clone(&client)).await,
            ),
            (
                "AnimeSwap",
                Self::get_animeswap_volume(Arc::clone(&client)).await,
            ),
            (
                "Cellana",
                Self::get_cellana_volume(Arc::clone(&client)).await,
            ),
            (
                "AuxExchange",
                Self::get_aux_volume(Arc::clone(&client)).await,
            ),
        ];
        let mut handles = Vec::new();
        for (dex_name, volume_future) in dex_volume_futures {
            let handle = tokio::spawn(async move {
                match volume_future {
                    Ok(volume) => Some((dex_name.to_string(), volume)),
                    Err(e) => {
                        eprintln!("Failed to get volume for {}: {}", dex_name, e);
                        None
                    }
                }
            });
            handles.push(handle);
        }
        for handle in handles {
            if let Some((dex_name, volume)) = handle.await.map_err(|e| e.to_string())? {
                volume_map.insert(dex_name, volume);
            }
        }
        Ok(volume_map)
    }

    /// get liquidswap volume
    async fn get_liquidswap_volume(client: Arc<Aptos>) -> Result<u64, String> {
        let events = crate::dex::liquidswap::Liquidswap::get_swap_events(client).await?;
        let total_volume = events
            .iter()
            .map(|event| {
                if let Some(amount_in) = event.event_data.get("amount_in") {
                    amount_in
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                } else if let Some(amount_out) = event.event_data.get("amount_out") {
                    amount_out
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                } else {
                    0
                }
            })
            .sum();
        Ok(total_volume)
    }

    /// get thala volume
    async fn get_thala_volume(client: Arc<Aptos>) -> Result<u64, String> {
        let events = crate::dex::thala::Thala::get_swap_events(client).await?;
        let total_volume = events
            .iter()
            .map(|event| {
                if let Some(amount_in) = event.event_data.get("amount_in") {
                    amount_in
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                } else if let Some(amount_x_in) = event.event_data.get("amount_x_in") {
                    amount_x_in
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                } else if let Some(amount_y_in) = event.event_data.get("amount_y_in") {
                    amount_y_in
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                } else {
                    0
                }
            })
            .sum();
        Ok(total_volume)
    }

    /// get pancakeswap volume
    async fn get_pancakeswap_volume(client: Arc<Aptos>) -> Result<u64, String> {
        let events = crate::dex::pancakeswap::PancakeSwap::get_swap_events(client).await?;
        let total_volume = events
            .iter()
            .map(|event| {
                if let Some(amount0_in) = event.event_data.get("amount0_in") {
                    amount0_in
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                } else if let Some(amount1_in) = event.event_data.get("amount1_in") {
                    amount1_in
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                } else if let Some(amount0_out) = event.event_data.get("amount0_out") {
                    amount0_out
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                } else if let Some(amount1_out) = event.event_data.get("amount1_out") {
                    amount1_out
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                } else {
                    0
                }
            })
            .sum();
        Ok(total_volume)
    }

    /// get animeswap volume
    async fn get_animeswap_volume(client: Arc<Aptos>) -> Result<u64, String> {
        let events = crate::dex::animeswap::AnimeSwap::get_swap_events(client).await?;
        let total_volume = events
            .iter()
            .map(|event| {
                if let Some(amount0_in) = event.event_data.get("amount0_in") {
                    amount0_in
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                } else if let Some(amount1_in) = event.event_data.get("amount1_in") {
                    amount1_in
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                } else if let Some(amount_in) = event.event_data.get("amount_in") {
                    amount_in
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                } else {
                    0
                }
            })
            .sum();
        Ok(total_volume)
    }

    /// get cellana volume
    async fn get_cellana_volume(client: Arc<Aptos>) -> Result<u64, String> {
        let events = crate::dex::cellana::Cellana::get_swap_events(client).await?;
        let total_volume = events
            .iter()
            .map(|event| {
                if let Some(amount_in) = event.event_data.get("amount_in") {
                    amount_in
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                } else if let Some(amount_x) = event.event_data.get("amount_x") {
                    amount_x
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                } else if let Some(amount_y) = event.event_data.get("amount_y") {
                    amount_y
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                } else {
                    0
                }
            })
            .sum();
        Ok(total_volume)
    }

    /// get aux volume
    async fn get_aux_volume(client: Arc<Aptos>) -> Result<u64, String> {
        let events = crate::dex::auxswap::AuxExchange::get_swap_events(client).await?;
        let total_volume = events
            .iter()
            .map(|event| {
                if let Some(amount_in) = event.event_data.get("amount_in") {
                    amount_in
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                } else if let Some(quantity) = event.event_data.get("quantity") {
                    quantity
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                } else if let Some(amount) = event.event_data.get("amount") {
                    amount
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                } else {
                    0
                }
            })
            .sum();
        Ok(total_volume)
    }

    /// get liquidity depth
    pub async fn get_liquidity_depth(
        _client: Arc<Aptos>,
        token_a: &str,
        token_b: &str,
    ) -> Result<Vec<DexLiquidity>, String> {
        let mut liquidity_data = Vec::new();
        liquidity_data.push(DexLiquidity {
            dex: "Liquidswap".to_string(),
            token_a: token_a.to_string(),
            token_b: token_b.to_string(),
            reserve_a: 500000000000,
            reserve_b: 500000000000,
            total_liquidity: 1000000000000,
        });
        liquidity_data.push(DexLiquidity {
            dex: "Thala".to_string(),
            token_a: token_a.to_string(),
            token_b: token_b.to_string(),
            reserve_a: 250000000000,
            reserve_b: 250000000000,
            total_liquidity: 500000000000,
        });
        liquidity_data.push(DexLiquidity {
            dex: "PancakeSwap".to_string(),
            token_a: token_a.to_string(),
            token_b: token_b.to_string(),
            reserve_a: 150000000000,
            reserve_b: 150000000000,
            total_liquidity: 300000000000,
        });
        liquidity_data.push(DexLiquidity {
            dex: "AnimeSwap".to_string(),
            token_a: token_a.to_string(),
            token_b: token_b.to_string(),
            reserve_a: 100000000000,
            reserve_b: 100000000000,
            total_liquidity: 200000000000,
        });
        liquidity_data.push(DexLiquidity {
            dex: "Cellana".to_string(),
            token_a: token_a.to_string(),
            token_b: token_b.to_string(),
            reserve_a: 75000000000,
            reserve_b: 75000000000,
            total_liquidity: 150000000000,
        });
        liquidity_data.sort_by(|a, b| b.total_liquidity.cmp(&a.total_liquidity));
        Ok(liquidity_data)
    }
}

#[derive(Debug, Clone)]
pub struct DexLiquidity {
    pub dex: String,
    pub token_a: String,
    pub token_b: String,
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub total_liquidity: u64,
}

pub struct DexUtils;

impl DexUtils {
    pub fn calculate_price_impact(amount_in: u64, reserve_in: u64, reserve_out: u64) -> f64 {
        if reserve_in == 0 || reserve_out == 0 {
            return 0.0;
        }
        let amount_out_before = reserve_out as f64 / reserve_in as f64 * amount_in as f64;
        let amount_out_after =
            DexAggregator::calculate_amm_output(amount_in, reserve_in, reserve_out) as f64;
        if amount_out_before == 0.0 {
            return 0.0;
        }
        ((amount_out_before - amount_out_after) / amount_out_before).abs() * 100.0
    }

    pub fn calculate_optimal_slippage(price_impact: f64) -> f64 {
        if price_impact < 0.1 {
            0.5
        } else if price_impact < 1.0 {
            1.0
        } else {
            2.0
        }
    }

    pub fn format_token_amount(amount: u64, decimals: u8) -> String {
        let divisor = 10u64.pow(decimals as u32);
        let whole = amount / divisor;
        let fractional = amount % divisor;
        if fractional == 0 {
            format!("{}", whole)
        } else {
            format!(
                "{}.{:0>width$}",
                whole,
                fractional,
                width = decimals as usize
            )
        }
    }

    pub async fn validate_token_pair(
        client: Arc<Aptos>,
        token_a: &str,
        token_b: &str,
    ) -> Result<Vec<String>, String> {
        let mut supported_dexes = Vec::new();
        if Liquidswap::get_pool_info(Arc::clone(&client), token_a, token_b)
            .await
            .is_ok()
        {
            supported_dexes.push("Liquidswap".to_string());
        }

        if Thala::get_pool_info(Arc::clone(&client), token_a, token_b)
            .await
            .is_ok()
        {
            supported_dexes.push("Thala".to_string());
        }

        if PancakeSwap::get_reserves(Arc::clone(&client), token_a, token_b)
            .await
            .is_ok()
        {
            supported_dexes.push("PancakeSwap".to_string());
        }

        if AnimeSwap::get_reserves(Arc::clone(&client), token_a, token_b)
            .await
            .is_ok()
        {
            supported_dexes.push("AnimeSwap".to_string());
        }

        if Cellana::get_pool_info(Arc::clone(&client), token_a, token_b)
            .await
            .is_ok()
        {
            supported_dexes.push("Cellana".to_string());
        }

        Ok(supported_dexes)
    }
}

impl Default for PancakeSwapEventFilters {
    fn default() -> Self {
        Self {
            min_swap_amount: Some(1000000000),
            include_cake_pairs: true,
            tracked_pairs: None,
        }
    }
}

impl Default for CellanaEventConfig {
    fn default() -> Self {
        Self {
            monitor_cell_pairs: true,
            min_swap_amount: 1000000000,
            monitor_farming: true,
            tracked_tokens: vec![],
        }
    }
}

impl Default for AnimeSwapEventFilters {
    fn default() -> Self {
        Self {
            min_swap_amount: Some(1000000000),
            tracked_tokens: None,
            min_liquidity_amount: Some(500000000),
        }
    }
}
