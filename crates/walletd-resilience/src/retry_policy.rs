//! Retry policies for different error types
//!
//! Determines which errors should trigger retries and how.

use std::time::Duration;

/// Trait for classifying errors as retryable or not
pub trait RetryClassifier<E> {
    /// Check if the error is retryable
    fn is_retryable(&self, error: &E) -> bool;
    
    /// Get suggested delay override for this error (if any)
    fn suggested_delay(&self, error: &E) -> Option<Duration>;
}

/// Default retry classifier for common error patterns
#[derive(Debug, Clone, Default)]
pub struct DefaultRetryClassifier;

impl<E: std::error::Error> RetryClassifier<E> for DefaultRetryClassifier {
    fn is_retryable(&self, error: &E) -> bool {
        let msg = error.to_string().to_lowercase();
        
        // Network/connection errors - always retry
        if msg.contains("connection") 
            || msg.contains("timeout")
            || msg.contains("timed out")
            || msg.contains("network")
            || msg.contains("dns")
            || msg.contains("resolve")
        {
            return true;
        }
        
        // Server errors - usually retry
        if msg.contains("500")
            || msg.contains("502")
            || msg.contains("503")
            || msg.contains("504")
            || msg.contains("internal server")
            || msg.contains("service unavailable")
            || msg.contains("gateway")
        {
            return true;
        }
        
        // Rate limiting - retry with backoff
        if msg.contains("429")
            || msg.contains("rate limit")
            || msg.contains("too many requests")
            || msg.contains("throttl")
        {
            return true;
        }
        
        // Temporary errors
        if msg.contains("temporary")
            || msg.contains("retry")
            || msg.contains("try again")
        {
            return true;
        }
        
        false
    }
    
    fn suggested_delay(&self, error: &E) -> Option<Duration> {
        let msg = error.to_string().to_lowercase();
        
        // Rate limiting usually needs longer delays
        if msg.contains("429") || msg.contains("rate limit") {
            return Some(Duration::from_secs(5));
        }
        
        None
    }
}

/// HTTP-specific retry classifier
#[derive(Debug, Clone, Default)]
pub struct HttpRetryClassifier;

impl HttpRetryClassifier {
    /// Check if HTTP status code is retryable
    pub fn is_status_retryable(status: u16) -> bool {
        matches!(
            status,
            408 | // Request Timeout
            425 | // Too Early
            429 | // Too Many Requests
            500 | // Internal Server Error
            502 | // Bad Gateway
            503 | // Service Unavailable
            504   // Gateway Timeout
        )
    }
    
    /// Check if status indicates rate limiting
    pub fn is_rate_limited(status: u16) -> bool {
        status == 429
    }
    
    /// Get retry delay from Retry-After header value
    pub fn parse_retry_after(value: &str) -> Option<Duration> {
        // Try to parse as seconds
        if let Ok(secs) = value.parse::<u64>() {
            return Some(Duration::from_secs(secs));
        }
        
        // Could also parse HTTP-date format here
        None
    }
}

/// RPC-specific retry classifier
#[derive(Debug, Clone, Default)]
pub struct RpcRetryClassifier;

impl RpcRetryClassifier {
    /// Check if RPC error code is retryable
    pub fn is_code_retryable(code: i64) -> bool {
        matches!(
            code,
            -32099..=-32000 | // Server errors
            -32603          | // Internal error
            -32005            // Limit exceeded
        )
    }
    
    /// Common retryable RPC error messages
    pub fn is_message_retryable(message: &str) -> bool {
        let msg = message.to_lowercase();
        msg.contains("nonce too low")
            || msg.contains("replacement transaction")
            || msg.contains("already known")
            || msg.contains("transaction pool")
            || msg.contains("pending")
    }
}

/// Blockchain-specific retry policies
#[derive(Debug, Clone)]
pub struct BlockchainRetryPolicy {
    /// Retry on nonce errors
    pub retry_nonce_errors: bool,
    /// Retry on mempool errors
    pub retry_mempool_errors: bool,
    /// Retry on gas estimation failures
    pub retry_gas_errors: bool,
    /// Maximum retries for transaction submission
    pub max_tx_retries: u32,
}

impl Default for BlockchainRetryPolicy {
    fn default() -> Self {
        Self {
            retry_nonce_errors: true,
            retry_mempool_errors: true,
            retry_gas_errors: true,
            max_tx_retries: 3,
        }
    }
}

impl BlockchainRetryPolicy {
    /// Create policy for Ethereum-like chains
    pub fn ethereum() -> Self {
        Self {
            retry_nonce_errors: true,
            retry_mempool_errors: true,
            retry_gas_errors: true,
            max_tx_retries: 3,
        }
    }
    
    /// Create policy for Bitcoin-like chains
    pub fn bitcoin() -> Self {
        Self {
            retry_nonce_errors: false, // No nonces in Bitcoin
            retry_mempool_errors: true,
            retry_gas_errors: false, // No gas in Bitcoin
            max_tx_retries: 2,
        }
    }
    
    /// Check if error message indicates retryable blockchain error
    pub fn is_retryable(&self, error: &str) -> bool {
        let msg = error.to_lowercase();
        
        if self.retry_nonce_errors && 
            (msg.contains("nonce") || msg.contains("sequence")) {
            return true;
        }
        
        if self.retry_mempool_errors &&
            (msg.contains("mempool") || msg.contains("pool") || msg.contains("pending")) {
            return true;
        }
        
        if self.retry_gas_errors &&
            (msg.contains("gas") || msg.contains("fee")) {
            return true;
        }
        
        false
    }
}

/// Combined retry policy
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum total attempts
    pub max_attempts: u32,
    /// Initial backoff delay
    pub initial_delay: Duration,
    /// Maximum backoff delay
    pub max_delay: Duration,
    /// Whether to retry on timeout
    pub retry_on_timeout: bool,
    /// Whether to retry on connection errors
    pub retry_on_connection_error: bool,
    /// Whether to retry on server errors (5xx)
    pub retry_on_server_error: bool,
    /// Whether to retry on rate limiting
    pub retry_on_rate_limit: bool,
    /// Custom retryable error codes
    pub retryable_codes: Vec<i64>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            retry_on_timeout: true,
            retry_on_connection_error: true,
            retry_on_server_error: true,
            retry_on_rate_limit: true,
            retryable_codes: vec![],
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set maximum attempts
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }
    
    /// Set initial delay
    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }
    
    /// Set maximum delay
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }
    
    /// Add retryable error code
    pub fn with_retryable_code(mut self, code: i64) -> Self {
        self.retryable_codes.push(code);
        self
    }
    
    /// Create a strict policy (fewer retries)
    pub fn strict() -> Self {
        Self {
            max_attempts: 2,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(5),
            retry_on_timeout: true,
            retry_on_connection_error: true,
            retry_on_server_error: false,
            retry_on_rate_limit: true,
            retryable_codes: vec![],
        }
    }
    
    /// Create a lenient policy (more retries)
    pub fn lenient() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            retry_on_timeout: true,
            retry_on_connection_error: true,
            retry_on_server_error: true,
            retry_on_rate_limit: true,
            retryable_codes: vec![],
        }
    }
    
    /// Check if an RPC error code should be retried
    pub fn should_retry_code(&self, code: i64) -> bool {
        self.retryable_codes.contains(&code) || RpcRetryClassifier::is_code_retryable(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Debug)]
    struct TestError(String);
    
    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    
    impl std::error::Error for TestError {}
    
    #[test]
    fn test_default_classifier_timeout() {
        let classifier = DefaultRetryClassifier;
        assert!(classifier.is_retryable(&TestError("connection timeout".to_string())));
        assert!(classifier.is_retryable(&TestError("request timed out".to_string())));
    }
    
    #[test]
    fn test_default_classifier_server_errors() {
        let classifier = DefaultRetryClassifier;
        assert!(classifier.is_retryable(&TestError("500 internal server error".to_string())));
        assert!(classifier.is_retryable(&TestError("503 service unavailable".to_string())));
    }
    
    #[test]
    fn test_default_classifier_rate_limit() {
        let classifier = DefaultRetryClassifier;
        assert!(classifier.is_retryable(&TestError("429 too many requests".to_string())));
        
        let delay = classifier.suggested_delay(&TestError("rate limit exceeded".to_string()));
        assert!(delay.is_some());
        assert!(delay.unwrap() >= Duration::from_secs(1));
    }
    
    #[test]
    fn test_default_classifier_non_retryable() {
        let classifier = DefaultRetryClassifier;
        assert!(!classifier.is_retryable(&TestError("invalid address".to_string())));
        assert!(!classifier.is_retryable(&TestError("insufficient funds".to_string())));
    }
    
    #[test]
    fn test_http_status_retryable() {
        assert!(HttpRetryClassifier::is_status_retryable(500));
        assert!(HttpRetryClassifier::is_status_retryable(502));
        assert!(HttpRetryClassifier::is_status_retryable(503));
        assert!(HttpRetryClassifier::is_status_retryable(429));
        
        assert!(!HttpRetryClassifier::is_status_retryable(200));
        assert!(!HttpRetryClassifier::is_status_retryable(400));
        assert!(!HttpRetryClassifier::is_status_retryable(404));
    }
    
    #[test]
    fn test_http_rate_limited() {
        assert!(HttpRetryClassifier::is_rate_limited(429));
        assert!(!HttpRetryClassifier::is_rate_limited(500));
    }
    
    #[test]
    fn test_retry_after_parsing() {
        assert_eq!(
            HttpRetryClassifier::parse_retry_after("60"),
            Some(Duration::from_secs(60))
        );
        assert!(HttpRetryClassifier::parse_retry_after("invalid").is_none());
    }
    
    #[test]
    fn test_rpc_code_retryable() {
        assert!(RpcRetryClassifier::is_code_retryable(-32000));
        assert!(RpcRetryClassifier::is_code_retryable(-32603));
        
        assert!(!RpcRetryClassifier::is_code_retryable(-32600)); // Invalid request
        assert!(!RpcRetryClassifier::is_code_retryable(-32601)); // Method not found
    }
    
    #[test]
    fn test_rpc_message_retryable() {
        assert!(RpcRetryClassifier::is_message_retryable("nonce too low"));
        assert!(RpcRetryClassifier::is_message_retryable("replacement transaction underpriced"));
        
        assert!(!RpcRetryClassifier::is_message_retryable("invalid signature"));
    }
    
    #[test]
    fn test_blockchain_policy_ethereum() {
        let policy = BlockchainRetryPolicy::ethereum();
        assert!(policy.is_retryable("nonce too low"));
        assert!(policy.is_retryable("transaction in mempool"));
        assert!(policy.is_retryable("gas price too low"));
    }
    
    #[test]
    fn test_blockchain_policy_bitcoin() {
        let policy = BlockchainRetryPolicy::bitcoin();
        assert!(!policy.is_retryable("nonce too low")); // No nonces
        assert!(policy.is_retryable("mempool full"));
    }
    
    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
        assert!(policy.retry_on_timeout);
        assert!(policy.retry_on_connection_error);
    }
    
    #[test]
    fn test_retry_policy_strict() {
        let policy = RetryPolicy::strict();
        assert_eq!(policy.max_attempts, 2);
        assert!(!policy.retry_on_server_error);
    }
    
    #[test]
    fn test_retry_policy_lenient() {
        let policy = RetryPolicy::lenient();
        assert_eq!(policy.max_attempts, 5);
        assert!(policy.retry_on_server_error);
    }
    
    #[test]
    fn test_retry_policy_custom_codes() {
        let policy = RetryPolicy::new()
            .with_retryable_code(-32001)
            .with_retryable_code(-32002);
        
        assert!(policy.should_retry_code(-32001));
        assert!(policy.should_retry_code(-32002));
        assert!(policy.should_retry_code(-32603)); // Default retryable
    }
}
