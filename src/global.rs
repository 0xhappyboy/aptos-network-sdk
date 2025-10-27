/// global parameters
pub mod rpc {
    /// aptos rpc url
    pub const APTOS_MAINNET_URL: &str = "https://fullnode.mainnet.aptoslabs.com/v1";
    pub const APTOS_TESTNET_URL: &str = "https://fullnode.testnet.aptoslabs.com/v1";
    pub const APTOS_DEVNET_URL: &str = "https://fullnode.devnet.aptoslabs.com/v1";
}
pub mod mainnet {
    /// system reserved address.
    pub mod sys_address {
        /// aptos framework, token standards, account management, time, etc.
        pub const X_1: &str = "0x1";
        pub const X_2: &str = "0x2";
        /// NFT / token standards
        pub const X_3: &str = "0x3";
        pub const X_4: &str = "0x4";
        pub const X_5: &str = "0x5";
    }
    /// system module
    pub mod sys_module {
        pub mod token {
            pub const name: &str = "token";
            pub const create_collection_script: &str = "create_collection_script";
            pub const create_token_script: &str = "create_token_script";
            pub const transfer_script: &str = "transfer_script";
            pub const collections: &str = "0x3::token::Collections";
            pub const token_store: &str = "0x3::token::TokenStore";
        }
        pub mod managed_coin {
            pub const name: &str = "managed_coin";
            pub const initialize: &str = "initialize";
            pub const mint: &str = "mint";
            pub const burn: &str = "burn";
            pub const register: &str = "register";
            pub const supply: &str = "supply";
        }
        pub mod coin {
            pub const name: &str = "coin";
            pub const create_currency: &str = "create_currency";
            pub const mint: &str = "mint";
            pub const burn: &str = "burn";
            pub const register: &str = "register";
            pub const supply: &str = "supply";
            pub const is_coin_initialized: &str = "is_coin_initialized";
            pub const is_account_registered: &str = "is_account_registered";
            pub const deposit: &str = "deposit";
            pub const withdraw: &str = "withdraw";
            pub const transfer: &str = "transfer";
            pub const balance: &str = "balance";
            pub const value: &str = "value";
            pub const zero: &str = "zero";
            pub const destroy_zero: &str = "destroy_zero";
        }
    }
    pub mod protocol_address {
        // thala protocol address
        pub const THALA_PROTOCOL_ADDRESS: &str =
            "0x7fd500c11216f0fe3095d0c4b8aa4d64a4e2e04f83758462f2b127255643615";
        // liquidswap protocol address
        pub const LIQUIDSWAP_PROTOCOL_ADDRESS: &str =
            "0x190d44266241744264b964a37b8f09863167a12d3e70cda39376cfb4e3561e12";
        // pancakeswap factory protocol address
        pub const PANCAKESWAP_FACTORY_PROTOCOL_ADDRESS: &str =
            "0xc7efb4076dbe143cbcd98cfaaa929ecfc8f299203dfff63b95ccb6bfe19850fa";
        // anime swap protocol address
        pub const ANIMESWAP_PROTOCOL_ADDRESS: &str =
            "0x16fe2df00ea7dde4a63409201f7f4e536bde7bb7335526a35d05111e68aa322c";
        // Aux swap protocol address
        pub const AUXSWAP_PROTOCOL_ADDRESS: &str =
            "0xbd35135844473187163ca197ca93b2ab014370587bb0ed3befff9e902d6bb541";
        // cellana swap protocol address
        pub const CELLANASWAP_PROTOCOL_ADDRESS: &str =
            "0x9b5a27d3e7c7c8f7f313f43e4bdc00d8b652b0c5e0e0e0e0e0e0e0e0e0e0e0e0";
    }
    pub mod nft_market {
        pub const TOPAZ: &'static str =
            "0x5c738c6a1fd8f29b7e5b6e6f79d6bd13d18083e6e3f3a43d1a8c5f3d6e6f79d6";
        pub const SOUFFL3: &'static str =
            "0x31f6d548c8e0b07ed82b4fd5377a61ddb064bb59e9a4c5e8e5e6f79d6bd13d18";
        pub const BLUEMOVE: &'static str =
            "0x6f5e58d4f7e8c3a9d4c5e8e5e6f79d6bd13d18083e6e3f3a43d1a8c5f3d6e6f79";
        pub const MERCATO: &'static str =
            "0x8c5f3d6e6f79d6bd13d18083e6e3f3a43d1a8c5f3d6e6f79d6bd13d18083e6e";
        pub const AUX_EXCHANGE: &'static str =
            "0xbd13d18083e6e3f3a43d1a8c5f3d6e6f79d6bd13d18083e6e3f3a43d1a8c5f3";
        pub const PANCAKE_SWAP_NFT: &'static str =
            "0x8e5e6f79d6bd13d18083e6e3f3a43d1a8c5f3d6e6f79d6bd13d18083e6e3f3a";
        pub const TRADEPORT: &'static str =
            "0x117f6a5d6e4c8f4d7e2c9c3d8b1a0e5c8a3b2d1c4e6f7a8b9c0d1e2f3a4b5c6d";
        pub const WAPAL: &'static str =
            "0x2a0c6a5d8e4f7b3c1d9e8a7b6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0a9b8c7";
    }
    pub mod token_address {
        pub const APT: &str = "0x1::aptos_coin::AptosCoin";
        pub const USDC: &str =
            "0x5e156f1207d0ebfa19a9eeff00d62a282278fb8719f4fab3a586a0a2c0fffbea::coin::T";
        pub const USDT: &str =
            "0x6f986d62e504433e05552cde45c4c6d9008ebafe47678d7f6a13ed8f6acd0e6::coin::T";
        pub const WORMHOLE_USDC: &str =
            "0xf22bede237a07e121b56d91a491eb7bcdfd1f5907926a9e58338f964a01b17fa::asset::USDC";
        pub const CAKE: &str =
            "0x159df6b7689437016108a019fd5bef736bac692b6d4a1f10c941f6fbb9a74ca6::oft::CakeOFT";
        pub const THL: &str =
            "0x7fd500c11216f0fe3095d0c4b8aa4d64a4e2e04f83758462f2b127255643615::thl_coin::THL";
    }
}
