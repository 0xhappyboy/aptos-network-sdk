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
        // pancakeswap protocol address
        pub const PANCAKESWAP_PROTOCOL_ADDRESS: &str =
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
    pub mod token_address {
        pub const USDC: &str = "0x5e156f1207d0ebfa19a9eeff00d62a282278fb8719f4fab3a586a0a2c0fffbea";
        pub const USDT: &str = "0x6f986d62e504433e05552cde45c4c6d6f8eebafe47678d7f6a13ed8f6acd0e6";
        pub const WORMHOLE_USDC: &str =
            "0xf22bede237a07e121b56d91a491eb7bcdfd1f5907926a9e58338f964a01b17fa";
    }
}
