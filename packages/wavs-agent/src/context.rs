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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Message;

    #[test]
    fn test_context_from_json() {
        // Valid context JSON
        let json = r#"{
            "contracts": [
                {
                    "name": "TestContract",
                    "address": "0x1234567890123456789012345678901234567890",
                    "abi": "[{\"name\":\"test\",\"type\":\"function\",\"inputs\":[],\"outputs\":[]}]",
                    "description": "Test contract"
                }
            ],
            "llm_config": {
                "temperature": 0.7,
                "top_p": 0.9,
                "seed": 123,
                "max_tokens": 500,
                "context_window": 4096
            },
            "model": "test-model",
            "messages": [
                {
                    "role": "system",
                    "content": "Test system message"
                }
            ],
            "config": {
                "test_key": "test_value"
            }
        }"#;

        let context = Context::from_json(json).unwrap();

        // Verify loaded values
        assert_eq!(context.contracts.len(), 1);
        assert_eq!(context.contracts[0].name, "TestContract");
        assert_eq!(context.contracts[0].address, "0x1234567890123456789012345678901234567890");
        assert_eq!(context.model, "test-model");
        assert_eq!(context.llm_config.temperature, 0.7);
        assert_eq!(context.llm_config.top_p, 0.9);
        assert_eq!(context.llm_config.seed, 123);
        assert_eq!(context.llm_config.max_tokens, Some(500));
        assert_eq!(context.llm_config.context_window, Some(4096));
        assert_eq!(context.messages.len(), 1);
        assert_eq!(context.messages[0].role, "system");
        assert_eq!(context.messages[0].content.as_ref().unwrap(), "Test system message");
        assert_eq!(context.config.get("test_key").unwrap(), "test_value");
    }

    #[test]
    fn test_context_validation() {
        // Valid context
        let valid_context = Context {
            contracts: vec![Contract::new(
                "TestContract",
                "0x1234567890123456789012345678901234567890",
                "[{\"name\":\"test\",\"type\":\"function\",\"inputs\":[],\"outputs\":[]}]",
            )],
            llm_config: LLMConfig::default(),
            model: "test-model".to_string(),
            messages: vec![Message::new_system("Test system message".to_string())],
            config: std::collections::HashMap::new(),
        };

        assert!(valid_context.validate().is_ok());

        // Invalid contract address
        let invalid_address_context = Context {
            contracts: vec![Contract::new(
                "TestContract",
                "invalid-address",
                "[{\"name\":\"test\",\"type\":\"function\",\"inputs\":[],\"outputs\":[]}]",
            )],
            llm_config: LLMConfig::default(),
            model: "test-model".to_string(),
            messages: vec![],
            config: std::collections::HashMap::new(),
        };

        assert!(invalid_address_context.validate().is_err());

        // Empty ABI
        let empty_abi_context = Context {
            contracts: vec![Contract::new(
                "TestContract",
                "0x1234567890123456789012345678901234567890",
                "",
            )],
            llm_config: LLMConfig::default(),
            model: "test-model".to_string(),
            messages: vec![],
            config: std::collections::HashMap::new(),
        };

        assert!(empty_abi_context.validate().is_err());
    }

    #[test]
    fn test_get_contract_by_name() {
        let context = Context {
            contracts: vec![
                Contract::new(
                    "Contract1",
                    "0x1111111111111111111111111111111111111111",
                    "[{\"name\":\"test\",\"type\":\"function\",\"inputs\":[],\"outputs\":[]}]",
                ),
                Contract::new(
                    "Contract2",
                    "0x2222222222222222222222222222222222222222",
                    "[{\"name\":\"test\",\"type\":\"function\",\"inputs\":[],\"outputs\":[]}]",
                ),
            ],
            llm_config: LLMConfig::default(),
            model: "test-model".to_string(),
            messages: vec![],
            config: std::collections::HashMap::new(),
        };

        // Test exact match
        let contract = context.get_contract_by_name("Contract1");
        assert!(contract.is_some());
        assert_eq!(contract.unwrap().name, "Contract1");

        // Test case insensitive match
        let contract = context.get_contract_by_name("contract2");
        assert!(contract.is_some());
        assert_eq!(contract.unwrap().name, "Contract2");

        // Test non-existent contract
        let contract = context.get_contract_by_name("NonExistentContract");
        assert!(contract.is_none());
    }

    #[test]
    fn test_format_contract_descriptions() {
        let context = Context {
            contracts: vec![
                Contract::new_with_description(
                    "Contract1",
                    "0x1111111111111111111111111111111111111111",
                    "[{\"name\":\"test\",\"type\":\"function\",\"inputs\":[],\"outputs\":[]}]",
                    "First test contract",
                ),
                Contract::new_with_description(
                    "Contract2",
                    "0x2222222222222222222222222222222222222222",
                    "[{\"name\":\"test\",\"type\":\"function\",\"inputs\":[],\"outputs\":[]}]",
                    "Second test contract",
                ),
            ],
            llm_config: LLMConfig::default(),
            model: "test-model".to_string(),
            messages: vec![],
            config: std::collections::HashMap::new(),
        };

        let descriptions = context.format_contract_descriptions();

        // Check that the descriptions contain the contract names, addresses, and ABIs
        assert!(descriptions.contains("Contract1"));
        assert!(descriptions.contains("0x1111111111111111111111111111111111111111"));
        assert!(descriptions.contains("Contract2"));
        assert!(descriptions.contains("0x2222222222222222222222222222222222222222"));

        // Check that descriptions are separated
        assert!(descriptions.contains("\n\n"));
    }

    #[test]
    fn test_default_context() {
        let context = Context::default();

        // Check that default context has reasonable values
        assert!(!context.contracts.is_empty());
        assert_eq!(context.model, "llama3.2");
        assert!(!context.messages.is_empty());
        assert_eq!(context.messages[0].role, "system");
        assert!(context.messages[0].content.is_some());
    }
}
