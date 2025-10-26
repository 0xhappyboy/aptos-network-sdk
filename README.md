<h1 align="center">
    Aptos Network SDK
</h1>
<h4 align="center">
Implement most of the commonly used practical transaction-related functions of the Aptos network.
</h4>
<p align="center">
  <a href="https://github.com/0xhappyboy/aptos-network-sdk/LICENSE"><img src="https://img.shields.io/badge/License-GPL3.0-d1d1f6.svg?style=flat&labelColor=1C2C2E&color=BEC5C9&logo=googledocs&label=license&logoColor=BEC5C9" alt="License"></a>
</p>
<p align="center">
<a href="./README_zh-CN.md">简体中文</a> | <a href="./README.md">English</a>
</p>

## Depend

```bash
cargo add aptos-network-sdk
```

## Quick Start

```rust
use aptos_sdk::{AptosClient, AptosClientType, Wallet, Contract};
// Create client and wallet
let client = Arc::new(AptosClient::new(AptosClientType::Devnet));
let wallet = Arc::new(Wallet::new().unwrap());

// Get account balance
let balance = client.get_apt_balance_by_account(&wallet.address().unwrap()).await?;
println!("Balance: {} APT", balance);

// Transfer APT
let recipient = "0x123...";
let result = Contract::transfer_apt(
    Arc::clone(&client),
    Arc::clone(&wallet),
    recipient,
    100_000_000, // 1 APT
).await?;
println!("Transfer hash: {}", result.transaction_hash);

// Read contract data
let call = ContractCall {
    module_address: "0x1".to_string(),
    module_name: "coin".to_string(),
    function_name: "balance".to_string(),
    type_arguments: vec!["0x1::aptos_coin::AptosCoin".to_string()],
    arguments: vec![json!(wallet.address().unwrap())],
};
let result = Contract::read(Arc::clone(&client), &call).await?;
println!("Contract result: {:?}", result.data);
```

## Token

### Creating and registering new tokens

```rust
 use crate::token::{TokenManager, TokenUtils};
 use std::sync::Arc;
 use crate::{
    global::rpc::{APTOS_MAINNET_URL},
    types::*,
 };

 async fn create_new_token() -> Result<(), String> {
 let client = Arc::new(AptosClient::new(APTOS_MAINNET_URL));
 let wallet = Arc::new(Wallet::from_private_key("your_private_key"));
 let result = TokenManager::create_token(
     client.clone(),
     wallet.clone(),
     "My Token",
     "MYT",
     8,
     1_000_000_000,
 ).await?;

 let token_type = TokenUtils::build_standard_token_type(
     &wallet.address(),
     "my_token",
     "MYT"
 );
 let register_result = TokenManager::register_token(
     client.clone(),
     wallet.clone(),
     &token_type,
 ).await?;
 Ok(())
}
```

### Token minting and balance inquiry

```rust
use crate::{
    global::rpc::{APTOS_MAINNET_URL},
};

async fn mint_and_check_balance() -> Result<(), String> {
let client = Arc::new(AptosClient::new(APTOS_MAINNET_URL));
let wallet = Arc::new(Wallet::from_private_key("your_private_key"));
    let token_type = "0x1::managed_coin::MYT";
    let recipient = "0x123...";
    let mint_result = TokenManager::mint_token(
        client.clone(),
        wallet.clone(),
        token_type,
        recipient,
        100_000_000,
    ).await?;
    let balance = TokenManager::get_token_balance(
        client.clone(),
        recipient,
        token_type,
    ).await?;
    Ok(())
}
```

### Token search function

```rust
use crate::token::{TokenSearchManager, TokenSearchResult};
use crate::{
    global::rpc::{APTOS_MAINNET_URL},
    types::*,
};

async fn search_tokens() -> Result<(), String> {
    let client = Arc::new(AptosClient::new(APTOS_MAINNET_URL));
    let results = TokenSearchManager::get_token_by_symbol(
        client.clone(),
        "USDC",
    ).await?;
    let top_tokens = TokenSearchManager::get_top_token_vec(client.clone()).await?;
    for token in top_tokens {
       //..
    }
    Ok(())
}
```

### Token tool usage

```rust
fn token_utils_examples() {
    let token_type = TokenUtils::build_standard_token_type("0x1234567890abcdef","my_collection","MYT");
    if let Some((creator, collection, name)) = TokenUtils::parse_token_type(&token_type) {
    }
    let address = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
}
```

### Get token trading pair information

```rust
use crate::{
    global::rpc::{APTOS_MAINNET_URL},
    types::*,
};

async fn get_trading_pairs() -> Result<(), String> {
let client = Arc::new(AptosClient::new(APTOS_MAINNET_URL));
    let token_address = "0x1::aptos_coin::AptosCoin";
    let trading_pairs = TokenSearchManager::get_token_trading_pairs(
        client.clone(),
        token_address,
    ).await?;
    for pair in trading_pairs {
       // ...
    }
    Ok(())
}
```

### Complete token management process

```rust
use crate::{
    global::rpc::{APTOS_MAINNET_URL},
    types::*,
};

async fn complete_token_lifecycle() -> Result<(), String> {
    let client = Arc::new(AptosClient::new(APTOS_MAINNET_URL));
    let wallet = Arc::new(Wallet::from_private_key("your_private_key"));
    TokenManager::create_token(
        client.clone(),
        wallet.clone(),
        "Test Token",
        "TEST",
        6,
        10_000_000,
    ).await?;
    let token_type = format!("{}::test_token::TEST", wallet.address());
    // Register token
    TokenManager::register_token(client.clone(), wallet.clone(), &token_type).await?;
    // mint token
    TokenManager::mint_token(
        client.clone(),
        wallet.clone(),
        &token_type,
        &wallet.address(),
        1_000_000,
    ).await?;
    // Query token metadata
    let metadata = TokenManager::get_token_metadata(client.clone(), &token_type).await?;
    // Check balance
    let balance = TokenManager::get_token_balance(
        client.clone(),
        &wallet.address(),
        &token_type,
    ).await?;
    Ok(())
}
```

## Event

### Basic event listener

```rust
use aptos_sdk::{AptosClient, AptosClientType, Contract, event::{EventHandler, EventSubscriptionManager}};

let client = Arc::new(AptosClient::new(AptosClientType::Mainnet));

// listener coin transfer events
Contract::listen_events_all_info(
    Arc::clone(&client),
    "0x1",
    "0x1::coin::WithdrawEvent",
    |result| {
        match result {
            Ok(event) => {
                println!("New Withdraw Event:");
                println!("Type: {}", event.r#type);
                println!("Sequence: {}", event.sequence_number);
                println!("Data: {:?}", event.data);

                // Extract amount from event data
                if let Some(amount) = event.data.get("amount") {
                    println!("Amount: {}", amount);
                }
            }
            Err(e) => eprintln!("Event monitoring error: {}", e),
        }
    },
    3, // check interval
).await?;
```

### Event Streaming

```rust
use tokio::sync::broadcast;

let mut event_manager = EventSubscriptionManager::new();
let receiver = event_manager.subscribe("coin_events".to_string());

// Start event stream in background
tokio::spawn(async move {
    let (sender, _) = broadcast::channel(100);
    if let Err(e) = EventHandler::start_event_stream(
        Arc::clone(&client),
        "0x1".to_string(),
        "0x1::coin::DepositEvent".to_string(),
        sender,
    ).await {
        eprintln!("Event stream failed: {}", e);
    }
});

// Process events from receiver
while let Ok(event_data) = receiver.recv().await {
    println!("Deposit detected!");
    println!("Block: {}", event_data.block_height);
    println!("TX Hash: {}", event_data.transaction_hash);

    if let Some(account) = event_data.event_data.get("account") {
        println!("Account: {}", account);
    }
}
```
