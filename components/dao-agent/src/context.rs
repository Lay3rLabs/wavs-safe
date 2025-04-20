use crate::contracts::{Contract, TokenBalance};

// TODO add LlmConfig to the context
// TODO add model to the context
// TODO add system prompt to the context
// TODO serialize and deserialize the context to a json string
/// Context for the DAO agent's decision making
#[derive(Debug)]
pub struct DaoContext {
    pub safe_address: String,
    pub eth_balance: TokenBalance,
    pub token_balances: Vec<TokenBalance>,
    pub allowed_addresses: Vec<String>,
    pub dao_description: String,
    pub contracts: Vec<Contract>,
}

// TODO make it so we don't have to hardcode these, fetch them from an IPFS
impl Default for DaoContext {
    fn default() -> Self {
        Self {
            safe_address: "0x742d35Cc6634C0532925a3b844Bc454e4438f44e".to_string(),
            eth_balance: TokenBalance::new(
                "0x0000000000000000000000000000000000000000",
                "ETH",
                "100000000000000000000", // 100 ETH in wei
                18,
            ),
            token_balances: vec![TokenBalance::new(
                "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                "USDC",
                "1000000000", // 1000 USDC (6 decimals)
                6,
            )],
            allowed_addresses: vec!["0xDf3679681B87fAE75CE185e4f01d98b64Ddb64a3".to_string()],
            dao_description: "A DAO focused on funding public goods and environmental causes"
                .to_string(),
            contracts: vec![Contract::new(
                "USDC",
                "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                r#"[{
                        "constant": false,
                        "inputs": [{"name": "_to","type": "address"},{"name": "_value","type": "uint256"}],
                        "name": "transfer",
                        "outputs": [{"name": "","type": "bool"}],
                        "type": "function"
                    }]"#,
            )],
        }
    }
}

impl DaoContext {
    /// Check if the context has sufficient ETH balance for a transaction
    pub fn has_sufficient_eth(&self, amount_wei: &str) -> bool {
        let amount = amount_wei.parse::<u128>().unwrap_or(0);
        let balance = self.eth_balance.balance.parse::<u128>().unwrap_or(0);
        amount <= balance
    }

    /// Check if a given token is supported and has sufficient balance
    pub fn has_sufficient_token_balance(&self, token_address: &str, amount: &str) -> bool {
        // Find the token balance
        if let Some(token) = self.token_balances.iter().find(|t| t.token_address == token_address) {
            let amount_val = amount.parse::<u128>().unwrap_or(u128::MAX);
            let balance_val = token.balance.parse::<u128>().unwrap_or(0);
            return amount_val <= balance_val;
        }
        false
    }

    /// Get a token by its symbol
    pub fn get_token_by_symbol(&self, symbol: &str) -> Option<&TokenBalance> {
        self.token_balances.iter().find(|t| t.symbol.to_lowercase() == symbol.to_lowercase())
    }

    /// Check if an address is in the allowed list
    pub fn is_address_allowed(&self, address: &str) -> bool {
        self.allowed_addresses.iter().any(|a| a.to_lowercase() == address.to_lowercase())
    }

    /// Get a smart contract by name
    pub fn get_contract_by_name(&self, name: &str) -> Option<&Contract> {
        self.contracts.iter().find(|c| c.name.to_lowercase() == name.to_lowercase())
    }

    /// Get list of supported token symbols
    pub fn get_supported_token_symbols(&self) -> Vec<String> {
        self.token_balances.iter().map(|t| t.symbol.clone()).collect()
    }

    /// Format balances for display in the prompt
    pub fn format_balances(&self) -> String {
        let mut result = vec![format!(
            "ETH: {} ({})",
            self.eth_balance.formatted_balance(),
            self.eth_balance.symbol
        )];

        for balance in &self.token_balances {
            result.push(format!(
                "{}: {} ({})",
                balance.symbol,
                balance.formatted_balance(),
                balance.token_address
            ));
        }

        result.join("\n")
    }
}
