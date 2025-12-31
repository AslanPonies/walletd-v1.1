use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionBuilder {
    pub operations: Vec<Operation>,
    pub nonce: Option<u64>,
    pub gas_limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    Transfer {
        to: String,
        amount: u128,
    },
    CreateObject {
        class_id: String,
        initial_state: serde_json::Value,
    },
    InvokeMethod {
        object_id: String,
        method: String,
        params: Vec<serde_json::Value>,
    },
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            nonce: None,
            gas_limit: None,
        }
    }

    pub fn add_operation(mut self, op: Operation) -> Self {
        self.operations.push(op);
        self
    }

    pub fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }

    pub fn with_gas_limit(mut self, gas_limit: u64) -> Self {
        self.gas_limit = Some(gas_limit);
        self
    }
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // TransactionBuilder Tests
    // ============================================================================

    #[test]
    fn test_new_builder() {
        let builder = TransactionBuilder::new();
        assert!(builder.operations.is_empty());
        assert!(builder.nonce.is_none());
        assert!(builder.gas_limit.is_none());
    }

    #[test]
    fn test_default_builder() {
        let builder = TransactionBuilder::default();
        assert!(builder.operations.is_empty());
    }

    #[test]
    fn test_add_transfer_operation() {
        let builder = TransactionBuilder::new()
            .add_operation(Operation::Transfer {
                to: "saga123".to_string(),
                amount: 1000,
            });
        
        assert_eq!(builder.operations.len(), 1);
        match &builder.operations[0] {
            Operation::Transfer { to, amount } => {
                assert_eq!(to, "saga123");
                assert_eq!(*amount, 1000);
            }
            _ => panic!("Expected Transfer operation"),
        }
    }

    #[test]
    fn test_add_create_object_operation() {
        let state = serde_json::json!({"name": "test", "value": 42});
        let builder = TransactionBuilder::new()
            .add_operation(Operation::CreateObject {
                class_id: "TestClass".to_string(),
                initial_state: state.clone(),
            });
        
        assert_eq!(builder.operations.len(), 1);
        match &builder.operations[0] {
            Operation::CreateObject { class_id, initial_state } => {
                assert_eq!(class_id, "TestClass");
                assert_eq!(initial_state, &state);
            }
            _ => panic!("Expected CreateObject operation"),
        }
    }

    #[test]
    fn test_add_invoke_method_operation() {
        let params = vec![serde_json::json!("arg1"), serde_json::json!(123)];
        let builder = TransactionBuilder::new()
            .add_operation(Operation::InvokeMethod {
                object_id: "obj_123".to_string(),
                method: "transfer".to_string(),
                params: params.clone(),
            });
        
        assert_eq!(builder.operations.len(), 1);
        match &builder.operations[0] {
            Operation::InvokeMethod { object_id, method, params: p } => {
                assert_eq!(object_id, "obj_123");
                assert_eq!(method, "transfer");
                assert_eq!(p, &params);
            }
            _ => panic!("Expected InvokeMethod operation"),
        }
    }

    #[test]
    fn test_with_nonce() {
        let builder = TransactionBuilder::new()
            .with_nonce(42);
        
        assert_eq!(builder.nonce, Some(42));
    }

    #[test]
    fn test_with_gas_limit() {
        let builder = TransactionBuilder::new()
            .with_gas_limit(1_000_000);
        
        assert_eq!(builder.gas_limit, Some(1_000_000));
    }

    #[test]
    fn test_chaining_operations() {
        let builder = TransactionBuilder::new()
            .add_operation(Operation::Transfer {
                to: "saga123".to_string(),
                amount: 1000,
            })
            .add_operation(Operation::Transfer {
                to: "saga456".to_string(),
                amount: 2000,
            })
            .with_nonce(1)
            .with_gas_limit(500_000);
        
        assert_eq!(builder.operations.len(), 2);
        assert_eq!(builder.nonce, Some(1));
        assert_eq!(builder.gas_limit, Some(500_000));
    }

    #[test]
    fn test_multiple_operation_types() {
        let builder = TransactionBuilder::new()
            .add_operation(Operation::CreateObject {
                class_id: "Token".to_string(),
                initial_state: serde_json::json!({"supply": 1000000}),
            })
            .add_operation(Operation::InvokeMethod {
                object_id: "token_1".to_string(),
                method: "mint".to_string(),
                params: vec![serde_json::json!(100)],
            })
            .add_operation(Operation::Transfer {
                to: "saga789".to_string(),
                amount: 50,
            });
        
        assert_eq!(builder.operations.len(), 3);
    }

    // ============================================================================
    // Operation Serialization Tests
    // ============================================================================

    #[test]
    fn test_transfer_serialization() {
        let op = Operation::Transfer {
            to: "saga123".to_string(),
            amount: 1000,
        };
        
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("Transfer"));
        assert!(json.contains("saga123"));
        assert!(json.contains("1000"));
    }

    #[test]
    fn test_create_object_serialization() {
        let op = Operation::CreateObject {
            class_id: "MyClass".to_string(),
            initial_state: serde_json::json!({"field": "value"}),
        };
        
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("CreateObject"));
        assert!(json.contains("MyClass"));
    }

    #[test]
    fn test_invoke_method_serialization() {
        let op = Operation::InvokeMethod {
            object_id: "obj_1".to_string(),
            method: "update".to_string(),
            params: vec![serde_json::json!(true)],
        };
        
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("InvokeMethod"));
        assert!(json.contains("update"));
    }

    #[test]
    fn test_builder_serialization() {
        let builder = TransactionBuilder::new()
            .add_operation(Operation::Transfer {
                to: "saga123".to_string(),
                amount: 1000,
            })
            .with_nonce(5)
            .with_gas_limit(100000);
        
        let json = serde_json::to_string(&builder).unwrap();
        let deserialized: TransactionBuilder = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.operations.len(), 1);
        assert_eq!(deserialized.nonce, Some(5));
        assert_eq!(deserialized.gas_limit, Some(100000));
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[test]
    fn test_empty_operations() {
        let builder = TransactionBuilder::new()
            .with_nonce(0)
            .with_gas_limit(0);
        
        assert!(builder.operations.is_empty());
        assert_eq!(builder.nonce, Some(0));
        assert_eq!(builder.gas_limit, Some(0));
    }

    #[test]
    fn test_large_amount() {
        let builder = TransactionBuilder::new()
            .add_operation(Operation::Transfer {
                to: "saga123".to_string(),
                amount: u128::MAX,
            });
        
        match &builder.operations[0] {
            Operation::Transfer { amount, .. } => {
                assert_eq!(*amount, u128::MAX);
            }
            _ => panic!("Expected Transfer"),
        }
    }

    #[test]
    fn test_empty_params() {
        let builder = TransactionBuilder::new()
            .add_operation(Operation::InvokeMethod {
                object_id: "obj".to_string(),
                method: "noArgs".to_string(),
                params: vec![],
            });
        
        match &builder.operations[0] {
            Operation::InvokeMethod { params, .. } => {
                assert!(params.is_empty());
            }
            _ => panic!("Expected InvokeMethod"),
        }
    }
}
