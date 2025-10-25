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
