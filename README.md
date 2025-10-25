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
