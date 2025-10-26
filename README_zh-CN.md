<h1 align="center">
    Aptos Network SDK
</h1>
<h4 align="center">
实现Aptos网络大部分常用实用交易相关功能。
</h4>
<p align="center">
  <a href="https://github.com/0xhappyboy/aptos-network-sdk/LICENSE"><img src="https://img.shields.io/badge/License-GPL3.0-d1d1f6.svg?style=flat&labelColor=1C2C2E&color=BEC5C9&logo=googledocs&label=license&logoColor=BEC5C9" alt="License"></a>
</p>
<p align="center">
<a href="./README_zh-CN.md">简体中文</a> | <a href="./README.md">English</a>
</p>

## 快速启动

```rust
use aptos_sdk::{AptosClient, AptosClientType, Wallet, Contract};

// 创建客户端和钱包
let client = Arc::new(AptosClient::new(AptosClientType::Devnet));
let wallet = Arc::new(Wallet::new().unwrap());

// 查询账户余额
let balance = client.get_apt_balance_by_account(&wallet.address().unwrap()).await?;
println!("余额: {} APT", balance);

// 转账 APT
let recipient = "0x123...";
let result = Contract::transfer_apt(
    Arc::clone(&client),
    Arc::clone(&wallet),
    recipient,
    100_000_000, // 1 APT
).await?;
println!("转账哈希: {}", result.transaction_hash);

// 读取合约数据
let call = ContractCall {
    module_address: "0x1".to_string(),
    module_name: "coin".to_string(),
    function_name: "balance".to_string(),
    type_arguments: vec!["0x1::aptos_coin::AptosCoin".to_string()],
    arguments: vec![json!(wallet.address().unwrap())],
};
let result = Contract::read(Arc::clone(&client), &call).await?;
println!("合约结果: {:?}", result.data);
```

## 代币

### 创建和注册新代币

```rust
use crate::token::{TokenManager, TokenUtils};
use std::sync::Arc;
use crate::{
    global::rpc::{APTOS_MAINNET_URL},
    types::*,
};

// 创建新代币
async fn create_new_token() -> Result<(), String> {
let client = Arc::new(AptosClient::new(APTOS_MAINNET_URL));
let wallet = Arc::new(Wallet::from_private_key("your_private_key"));

 // 创建代币
 let result = TokenManager::create_token(
     client.clone(),
     wallet.clone(),
     "My Token",
     "MYT",
     8,
     1_000_000_000, // 10亿个代币，考虑小数位
 ).await?;

 println!("代币创建成功: {:?}", result);

 // 注册代币到当前账户
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

 println!("代币注册成功: {:?}", register_result);
 Ok(())

}
```

### 2. 代币铸造和余额查询

```rust
use crate::{
    global::rpc::{APTOS_MAINNET_URL},
};

// 铸造代币并查询余额
async fn mint_and_check_balance() -> Result<(), String> {
let client = Arc::new(AptosClient::new(APTOS_MAINNET_URL));
let wallet = Arc::new(Wallet::from_private_key("your_private_key"));
    let token_type = "0x1::managed_coin::MYT";
    let recipient = "0x123...";
    // 铸造代币
    let mint_result = TokenManager::mint_token(
        client.clone(),
        wallet.clone(),
        token_type,
        recipient,
        100_000_000, // 100个代币
    ).await?;
    println!("代币铸造成功: {:?}", mint_result);
    // 查询余额
    let balance = TokenManager::get_token_balance(
        client.clone(),
        recipient,
        token_type,
    ).await?;

    println!("账户 {} 的余额: {}", recipient, balance);
    Ok(())

}
```

### 代币搜索功能

```rust
use crate::token::{TokenSearchManager, TokenSearchResult};
use crate::{
    global::rpc::{APTOS_MAINNET_URL},
    types::*,
};

// 搜索代币
async fn search_tokens() -> Result<(), String> {
    let client = Arc::new(AptosClient::new(APTOS_MAINNET_URL));

    // 搜索 USDC 相关代币
    let results = TokenSearchManager::get_token_by_symbol(
        client.clone(),
        "USDC",
    ).await?;

    println!("找到 {} 个 USDC 相关代币:", results.len());
    for token in results {
        println!("符号: {}, 地址: {}, 已验证: {}",
            token.symbol, token.address, token.verified);
    }

    // 获取热门代币
    let top_tokens = TokenSearchManager::get_top_token_vec(client.clone()).await?;
    println!("热门代币:");
    for token in top_tokens {
        println!("{} - 价格: ${}, 24小时交易量: {}",
            token.symbol, token.price, token.volume_24h);
    }

    Ok(())

}
```

### 代币工具使用

```rust
// 代币工具使用示例
fn token_utils_examples() {
    // 构建标准代币类型
    let token_type = TokenUtils::build_standard_token_type("0x1234567890abcdef","my_collection","MYT");
    println!("代币类型: {}", token_type);
    // 解析代币类型
    if let Some((creator, collection, name)) = TokenUtils::parse_token_type(&token_type) {
        println!("创建者: {}, 集合: {}, 名称: {}", creator, collection, name);
    }
    // 验证地址格式
    let address = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
    println!("地址有效: {}", TokenUtils::is_valid_token_address(address));
}
```

### 获取代币交易对信息

```rust
use crate::{
    global::rpc::{APTOS_MAINNET_URL},
    types::*,
};

// 获取代币交易对
async fn get_trading_pairs() -> Result<(), String> {
let client = Arc::new(AptosClient::new(APTOS_MAINNET_URL));
    let token_address = "0x1::aptos_coin::AptosCoin";
    let trading_pairs = TokenSearchManager::get_token_trading_pairs(
        client.clone(),
        token_address,
    ).await?;
    println!("APT 交易对:");
    for pair in trading_pairs {
        println!("{} - {} | DEX: {:?} | 流动性: {}",
            pair.token_a, pair.token_b, pair.dexes, pair.total_liquidity);
    }
    Ok(())
}
```

### 完整的代币管理流程

```rust
use crate::{
    global::rpc::{APTOS_MAINNET_URL},
    types::*,
};

// 完整的代币创建和管理流程
async fn complete_token_lifecycle() -> Result<(), String> {
    let client = Arc::new(AptosClient::new(APTOS_MAINNET_URL));
    let wallet = Arc::new(Wallet::from_private_key("your_private_key"));
    // 创建代币
    println!("创建代币...");
    TokenManager::create_token(
        client.clone(),
        wallet.clone(),
        "Test Token",
        "TEST",
        6,
        10_000_000,
    ).await?;
    let token_type = format!("{}::test_token::TEST", wallet.address());
    // 注册代币
    println!("注册代币...");
    TokenManager::register_token(client.clone(), wallet.clone(), &token_type).await?;
    // 铸造代币
    println!("铸造代币...");
    TokenManager::mint_token(
        client.clone(),
        wallet.clone(),
        &token_type,
        &wallet.address(),
        1_000_000,
    ).await?;
    // 查询代币元数据
    println!("查询代币元数据...");
    let metadata = TokenManager::get_token_metadata(client.clone(), &token_type).await?;
    println!("代币元数据: {:?}", metadata);
    // 查询余额
    println!("查询余额...");
    let balance = TokenManager::get_token_balance(
        client.clone(),
        &wallet.address(),
        &token_type,
    ).await?;
    println!("当前余额: {}", balance);

    Ok(())
}
```

## 事件

### 基础事件监听

```rust
use aptos_sdk::{AptosClient, AptosClientType, Contract, event::{EventHandler, EventSubscriptionManager}};

let client = Arc::new(AptosClient::new(AptosClientType::Mainnet));

// 监听代币转账事件
Contract::listen_events_all_info(
    Arc::clone(&client),
    "0x1",
    "0x1::coin::WithdrawEvent",
    |result| {
        match result {
            Ok(event) => {
                println!("新的提款事件:");
                println!("类型: {}", event.r#type);
                println!("序列号: {}", event.sequence_number);
                println!("数据: {:?}", event.data);

                // 从事件数据中提取金额
                if let Some(amount) = event.data.get("amount") {
                    println!("金额: {}", amount);
                }
            }
            Err(e) => eprintln!("事件监听错误: {}", e),
        }
    },
    3, // 检查间隔
).await?;
```

### 事件流处理

```rust
use tokio::sync::broadcast;

let mut event_manager = EventSubscriptionManager::new();
let receiver = event_manager.subscribe("代币事件".to_string());

// 在后台启动事件流
tokio::spawn(async move {
    let (sender, _) = broadcast::channel(100);
    if let Err(e) = EventHandler::start_event_stream(
        Arc::clone(&client),
        "0x1".to_string(),
        "0x1::coin::DepositEvent".to_string(),
        sender,
    ).await {
        eprintln!("事件流启动失败: {}", e);
    }
});

// 从接收器处理事件
while let Ok(event_data) = receiver.recv().await {
    println!("检测到存款!");
    println!("区块: {}", event_data.block_height);
    println!("交易哈希: {}", event_data.transaction_hash);

    if let Some(account) = event_data.event_data.get("account") {
        println!("账户: {}", account);
    }
}
```
