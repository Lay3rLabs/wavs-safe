/// Represents a smart contract that the DAO can interact with
#[derive(Debug, Clone)]
pub struct Contract {
    pub name: String,
    pub address: String,
    pub abi: String, // JSON ABI string
}

/// Helper methods for working with contracts
impl Contract {
    /// Create a new Contract instance
    pub fn new(name: &str, address: &str, abi: &str) -> Self {
        Self { name: name.to_string(), address: address.to_string(), abi: abi.to_string() }
    }
}

/// Represents a token balance
#[derive(Debug, Clone)]
pub struct TokenBalance {
    pub token_address: String,
    pub symbol: String,
    pub balance: String,
    pub decimals: u8,
}

/// Helper methods for token balances
impl TokenBalance {
    /// Create a new TokenBalance instance
    pub fn new(token_address: &str, symbol: &str, balance: &str, decimals: u8) -> Self {
        Self {
            token_address: token_address.to_string(),
            symbol: symbol.to_string(),
            balance: balance.to_string(),
            decimals,
        }
    }

    /// Format the balance for display with proper decimal places
    pub fn formatted_balance(&self) -> String {
        let raw_balance = self.balance.parse::<u128>().unwrap_or(0);
        let divisor = 10u128.pow(self.decimals as u32);

        if divisor == 0 {
            return format!("{}", raw_balance);
        }

        let whole_part = raw_balance / divisor;
        let decimal_part = raw_balance % divisor;

        // Format with proper decimal places
        if decimal_part == 0 {
            format!("{}", whole_part)
        } else {
            // Pad decimal part with leading zeros if needed
            let decimal_str = format!("{:0width$}", decimal_part, width = self.decimals as usize);
            // Trim trailing zeros
            let trimmed = decimal_str.trim_end_matches('0');
            if trimmed.is_empty() {
                format!("{}", whole_part)
            } else {
                format!("{}.{}", whole_part, trimmed)
            }
        }
    }
}
