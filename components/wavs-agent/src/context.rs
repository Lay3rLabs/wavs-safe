use crate::contracts::Contract;
use crate::errors::{AgentError, AgentResult};
use crate::llm::LLMConfig;
use crate::tools::Message;
use serde::{Deserialize, Serialize};
use std::env;
use wavs_wasi_chain::http::{fetch_json, http_request_get};
use wstd::http::HeaderValue;

/// Generic context for agent's decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub contracts: Vec<Contract>,
    pub llm_config: LLMConfig,
    pub model: String,
    #[serde(default)]
    pub messages: Vec<Message>,
    /// Any global configuration values
    #[serde(default)]
    pub config: std::collections::HashMap<String, String>,
}

impl Context {
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
        let context: Context = fetch_json(req).await.unwrap();

        println!("Successfully loaded configuration");
        Ok(context)
    }

    /// Load context from JSON
    pub fn from_json(json: &str) -> AgentResult<Self> {
        let context: Self = serde_json::from_str(json).map_err(|e| {
            AgentError::Configuration(format!("Failed to parse context JSON: {}", e))
        })?;

        // Validate the context
        context.validate()?;

        Ok(context)
    }

    /// Serialize the context to a JSON string
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize context to JSON: {}", e))
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

    /// Get a smart contract by name
    pub fn get_contract_by_name(&self, name: &str) -> Option<&Contract> {
        self.contracts.iter().find(|c| c.name.to_lowercase() == name.to_lowercase())
    }

    /// Validate the Context for required fields and logical consistency
    pub fn validate(&self) -> AgentResult<()> {
        // Check each contract for required fields
        for (i, contract) in self.contracts.iter().enumerate() {
            if contract.address.is_empty() {
                return Err(AgentError::Configuration(format!(
                    "Contract at index {} is missing an address",
                    i
                )));
            }

            if contract.abi.is_empty() {
                return Err(AgentError::Configuration(format!(
                    "Contract at index {} is missing ABI",
                    i
                )));
            }

            // Validate contract address format
            if contract.address.len() != 42 || !contract.address.starts_with("0x") {
                return Err(AgentError::Configuration(format!(
                    "Contract at index {} has invalid address format: {}",
                    i, contract.address
                )));
            }
        }

        // Check for any required config items (none yet, but can be added)

        Ok(())
    }
}

// Default implementation for testing and development
impl Default for Context {
    fn default() -> Self {
        let default_system_prompt = r#"
            You are an agent responsible for making and executing financial transactions.
            
            You have several tools available to interact with smart contracts.
            Return nothing if no action is needed.
        "#
        .to_string();

        Self {
            contracts: vec![Contract::new_with_description(
                "USDC",
                "0xb7278a61aa25c888815afc32ad3cc52ff24fe575",
                r#"[{"type":"function","name":"transfer","inputs":[{"name":"to","type":"address","internalType":"address"},{"name":"value","type":"uint256","internalType":"uint256"}],"outputs":[{"name":"","type":"bool","internalType":"bool"}],"stateMutability":"nonpayable"}]"#,
                "USDC is a stablecoin pegged to the US Dollar",
            )],
            llm_config: LLMConfig::new()
                .temperature(0.0)
                .top_p(0.1)
                .seed(42)
                .max_tokens(Some(500))
                .context_window(Some(4096)),
            model: "llama3.2".to_string(),
            messages: vec![Message::new_system(default_system_prompt)],
            config: std::collections::HashMap::new(),
        }
    }
}
