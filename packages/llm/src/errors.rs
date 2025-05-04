use crate::bindings::exports::wavs::agent::errors::AgentError;

// Add additional error type for NotImplemented
impl AgentError {
    pub fn not_implemented(message: impl Into<String>) -> Self {
        AgentError::Other(message.into())
    }
}

// Convenience method for creating NotImplemented errors
pub fn not_implemented(message: impl Into<String>) -> AgentError {
    AgentError::not_implemented(message)
}
