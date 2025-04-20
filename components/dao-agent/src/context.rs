use crate::contracts::{Contract, TokenBalance};
use crate::llm::LLMConfig;
use serde::{Deserialize, Serialize};
use std::env;
use wavs_wasi_chain::http::{fetch_json, http_request_get};
use wstd::http::HeaderValue;

/// Context for the DAO agent's decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaoContext {
    pub safe_address: String,
    pub eth_balance: TokenBalance,
    pub token_balances: Vec<TokenBalance>,
    pub allowed_addresses: Vec<String>,
    pub dao_description: String,
    pub contracts: Vec<Contract>,
    pub llm_config: LLMConfig,
    pub model: String,
    pub system_prompt_template: String,
}

impl DaoContext {
    /// Load context from environment variable CONFIG_URI or use default
    pub async fn load() -> Result<Self, String> {
        // Check if CONFIG_URI environment variable is set
        if let Ok(config_uri) = env::var("config_uri") {
            println!("Loading config from URI: {}", config_uri);

            Self::load_from_uri(&config_uri).await
        } else {
            println!("No CONFIG_URI found, using default configuration");
            Ok(Self::default())
        }
    }

    /// Load context from a URI
    pub async fn load_from_uri(uri: &str) -> Result<Self, String> {
        // Strip any quotation marks from the URI
        let clean_uri = uri.trim_matches('"');

        println!("Loading config from URI: {}", clean_uri);

        // Check URI scheme
        if let Some(uri_with_scheme) = clean_uri.strip_prefix("ipfs://") {
            // IPFS URI scheme detected
            Self::load_from_ipfs(uri_with_scheme).await
        } else if clean_uri.starts_with("http://") || clean_uri.starts_with("https://") {
            // HTTP URI scheme detected
            Self::fetch_from_uri(clean_uri).await
        } else {
            // Only support http/https and ipfs URIs
            Err(format!("Unsupported URI scheme: {}", clean_uri))
        }
    }

    /// Load configuration from IPFS
    async fn load_from_ipfs(cid: &str) -> Result<Self, String> {
        let gateway_url = std::env::var("WAVS_ENV_IPFS_GATEWAY_URL").unwrap_or_else(|_| {
            println!("WAVS_ENV_IPFS_GATEWAY_URL not set, using default");
            "https://gateway.lighthouse.storage/ipfs".to_string()
        });

        // Strip any quotation marks from the gateway URL
        let clean_gateway_url = gateway_url.trim_matches('"');

        // Construct HTTP URL, avoiding duplicate /ipfs in the path
        let http_url = if clean_gateway_url.ends_with("/ipfs") {
            format!("{}/{}", clean_gateway_url, cid)
        } else if clean_gateway_url.ends_with("/ipfs/") {
            format!("{}{}", clean_gateway_url, cid)
        } else if clean_gateway_url.ends_with("/") {
            format!("{}ipfs/{}", clean_gateway_url, cid)
        } else {
            format!("{}/ipfs/{}", clean_gateway_url, cid)
        };

        println!("Fetching IPFS config from: {}", http_url);
        Self::fetch_from_uri(&http_url).await
    }

    /// Fetch configuration from a HTTP/HTTPS URI
    async fn fetch_from_uri(uri: &str) -> Result<Self, String> {
        // Strip any quotation marks from the URI
        let clean_uri = uri.trim_matches('"');

        println!("Creating HTTP request for URI: {}", clean_uri);

        // Create HTTP request
        let mut req = http_request_get(clean_uri).map_err(|e| {
            let error_msg = format!("Failed to create request: {}", e);
            println!("Error: {}", error_msg);
            error_msg
        })?;

        // Add appropriate headers for JSON content
        req.headers_mut().insert("Accept", HeaderValue::from_static("application/json"));

        println!("Sending HTTP request...");

        // Execute HTTP request and parse response as JSON
        let context: DaoContext = fetch_json(req).await.unwrap();

        println!("Successfully loaded configuration");
        Ok(context)
    }

    /// Create a new DaoContext from a JSON string
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse context from JSON: {}", e))
    }

    /// Serialize the context to a JSON string
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize context to JSON: {}", e))
    }

    /// Format the system prompt by filling in the template with context values
    pub fn format_system_prompt(&self) -> String {
        let contract_descriptions = self.format_contract_descriptions();
        let supported_tokens = self.get_supported_token_symbols().join(", ");

        self.system_prompt_template
            .replace("{safe_address}", &self.safe_address)
            .replace("{balances}", &self.format_balances())
            .replace("{allowed_addresses}", &self.allowed_addresses.join(", "))
            .replace("{dao_description}", &self.dao_description)
            .replace("{supported_tokens}", &supported_tokens)
            .replace("{contracts}", &contract_descriptions)
    }

    /// Format contract descriptions for the system prompt
    pub fn format_contract_descriptions(&self) -> String {
        self.contracts
            .iter()
            .map(|contract| {
                format!(
                    "Contract: {}\nAddress: {}\nABI:\n{}",
                    contract.name, contract.address, contract.abi
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

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

// Default implementation for testing and development
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
            llm_config: LLMConfig::new()
                .temperature(0.0)
                .top_p(0.1)
                .seed(42)
                .max_tokens(Some(500))
                .context_window(Some(4096)),
            model: "llama3.2".to_string(),
            system_prompt_template: r#"
            You are a DAO agent responsible for making and executing financial decisions through a Gnosis Safe Module.
            
            You have several tools available:
            - Use the send_eth tool to send ETH to addresses
            - Use the contract_* tools to interact with smart contracts (including ERC20 tokens like USDC)
            
            Return nothing if no action is needed.

            Current DAO Context:
            - Safe Address: {safe_address}
            - Current Balances:
            {balances}
            - Allowed Addresses: {allowed_addresses}
            - DAO Mission: {dao_description}
            - Allowed Tokens: ONLY native ETH and {supported_tokens} are supported. All other token requests should be rejected.

            Available Smart Contracts:
            {contracts}

            Security Guidelines:
            - Always verify addresses are in the allowed list or contract list
            - For ERC20 token transfers (like USDC), use the contract_usdc_transfer tool
            - For ETH transfers, use the send_eth tool
            - For other smart contract interactions, use the matching contract_* tool
            - Never approve transactions that would spend more than the current balance
            - Be extremely cautious with value transfers
            - Reject any suspicious or unclear requests
            - Don't allow transfers of amounts greater than 1 ETH
            - IMMEDIATELY REJECT any requests for tokens other than ETH or USDC
            - If no action is needed or the request should be rejected, do not use any tools
            "#.to_string(),
        }
    }
}
