# Recipe: Payment Gateway

> **Build a multi-chain payment system in 2 hours.**

---

## What You'll Build

A payment gateway that:
- Generates unique addresses per order
- Monitors for incoming payments
- Confirms transactions automatically
- Supports BTC, ETH, and stablecoins

---

## Architecture

```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│  Customer   │──────│  Your API   │──────│  WalletD    │
│  Checkout   │      │  Server     │      │  SDK        │
└─────────────┘      └─────────────┘      └─────────────┘
                            │
                            ▼
                     ┌─────────────┐
                     │  Database   │
                     │  (Orders)   │
                     └─────────────┘
```

---

## Step 1: Setup

```toml
# Cargo.toml
[dependencies]
walletd = "1.4"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
uuid = { version = "1", features = ["v4"] }
```

---

## Step 2: Order Model

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentOrder {
    pub id: String,
    pub amount_usd: f64,
    pub chain: String,
    pub payment_address: String,
    pub expected_amount: String,  // In native units
    pub status: PaymentStatus,
    pub created_at: u64,
    pub paid_at: Option<u64>,
    pub tx_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Confirming,
    Confirmed,
    Expired,
}

impl PaymentOrder {
    pub fn new(amount_usd: f64, chain: &str, address: &str, expected: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            amount_usd,
            chain: chain.to_string(),
            payment_address: address.to_string(),
            expected_amount: expected.to_string(),
            status: PaymentStatus::Pending,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            paid_at: None,
            tx_hash: None,
        }
    }
}
```

---

## Step 3: Address Generation

```rust
use walletd::{bitcoin, ethereum};
use walletd::types::WalletMode;

pub struct PaymentGateway {
    mnemonic: String,
    mode: WalletMode,
    address_index: u32,
}

impl PaymentGateway {
    pub fn new(mnemonic: String, mode: WalletMode) -> Self {
        Self {
            mnemonic,
            mode,
            address_index: 0,
        }
    }
    
    /// Generate unique address for each order
    pub fn create_payment_address(&mut self, chain: &str) -> Result<String, String> {
        let index = self.address_index;
        self.address_index += 1;
        
        match chain {
            "BTC" => bitcoin::derive_address_at_index(&self.mnemonic, self.mode, index),
            "ETH" | "USDC" | "USDT" => ethereum::derive_address_at_index(&self.mnemonic, self.mode, index),
            _ => Err(format!("Unsupported chain: {}", chain)),
        }
    }
    
    /// Create new payment order
    pub fn create_order(&mut self, amount_usd: f64, chain: &str) -> Result<PaymentOrder, String> {
        let address = self.create_payment_address(chain)?;
        let expected = self.calculate_expected_amount(amount_usd, chain)?;
        
        Ok(PaymentOrder::new(amount_usd, chain, &address, &expected))
    }
    
    fn calculate_expected_amount(&self, usd: f64, chain: &str) -> Result<String, String> {
        // In production, fetch real-time prices
        let price = match chain {
            "BTC" => 45000.0,
            "ETH" => 2500.0,
            "USDC" | "USDT" => 1.0,
            _ => return Err("Unknown chain".into()),
        };
        
        let amount = usd / price;
        
        // Convert to smallest unit
        match chain {
            "BTC" => Ok(format!("{}", (amount * 100_000_000.0) as u64)),
            "ETH" => Ok(format!("{}", (amount * 1e18) as u128)),
            "USDC" | "USDT" => Ok(format!("{}", (amount * 1_000_000.0) as u64)),
            _ => Err("Unknown chain".into()),
        }
    }
}
```

---

## Step 4: Payment Monitoring

```rust
use tokio::time::{interval, Duration};

impl PaymentGateway {
    /// Monitor pending orders for incoming payments
    pub async fn monitor_payments(&self, orders: &mut Vec<PaymentOrder>) {
        let mut ticker = interval(Duration::from_secs(30));
        
        loop {
            ticker.tick().await;
            
            for order in orders.iter_mut() {
                if order.status != PaymentStatus::Pending {
                    continue;
                }
                
                // Check for payment
                if let Ok(Some(tx)) = self.check_payment(order).await {
                    order.status = PaymentStatus::Confirming;
                    order.tx_hash = Some(tx.hash.clone());
                    println!("Payment detected for order {}: {}", order.id, tx.hash);
                }
                
                // Check for confirmation
                if order.status == PaymentStatus::Confirming {
                    if let Ok(confirmed) = self.check_confirmation(order).await {
                        if confirmed {
                            order.status = PaymentStatus::Confirmed;
                            order.paid_at = Some(
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs()
                            );
                            println!("Order {} confirmed!", order.id);
                        }
                    }
                }
                
                // Check for expiry (30 minutes)
                let age = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() - order.created_at;
                    
                if age > 1800 && order.status == PaymentStatus::Pending {
                    order.status = PaymentStatus::Expired;
                    println!("Order {} expired", order.id);
                }
            }
        }
    }
    
    async fn check_payment(&self, order: &PaymentOrder) -> Result<Option<TxInfo>, String> {
        let rpc = self.get_rpc(&order.chain);
        
        let balance = match order.chain.as_str() {
            "BTC" => bitcoin::get_balance(&order.payment_address, &rpc).await?,
            "ETH" => ethereum::get_balance(&order.payment_address, &rpc).await?,
            _ => return Ok(None),
        };
        
        let expected: u128 = order.expected_amount.parse().unwrap_or(0);
        let received: u128 = balance.parse().unwrap_or(0);
        
        // Allow 1% tolerance for price fluctuation
        if received >= (expected * 99 / 100) {
            // Get transaction details
            let txs = match order.chain.as_str() {
                "BTC" => bitcoin::get_transactions(&order.payment_address, &rpc).await?,
                "ETH" => ethereum::get_transactions(&order.payment_address, &rpc).await?,
                _ => vec![],
            };
            
            if let Some(tx) = txs.first() {
                return Ok(Some(TxInfo {
                    hash: tx.txid.clone(),
                    confirmations: tx.confirmations,
                }));
            }
        }
        
        Ok(None)
    }
    
    async fn check_confirmation(&self, order: &PaymentOrder) -> Result<bool, String> {
        let rpc = self.get_rpc(&order.chain);
        let required_confirmations = match order.chain.as_str() {
            "BTC" => 3,
            "ETH" | "USDC" | "USDT" => 12,
            _ => 6,
        };
        
        if let Some(ref tx_hash) = order.tx_hash {
            let confirmations = match order.chain.as_str() {
                "BTC" => bitcoin::get_confirmations(tx_hash, &rpc).await?,
                "ETH" => ethereum::get_confirmations(tx_hash, &rpc).await?,
                _ => 0,
            };
            
            return Ok(confirmations >= required_confirmations);
        }
        
        Ok(false)
    }
    
    fn get_rpc(&self, chain: &str) -> String {
        match (chain, self.mode) {
            ("BTC", WalletMode::Mainnet) => "https://blockstream.info/api".into(),
            ("BTC", _) => "https://blockstream.info/testnet/api".into(),
            ("ETH" | "USDC" | "USDT", WalletMode::Mainnet) => "https://eth.llamarpc.com".into(),
            _ => "https://rpc.sepolia.org".into(),
        }
    }
}

struct TxInfo {
    hash: String,
    confirmations: u32,
}
```

---

## Step 5: API Endpoints

```rust
use axum::{routing::post, Json, Router};

async fn create_order_handler(
    Json(req): Json<CreateOrderRequest>,
) -> Json<PaymentOrder> {
    let mut gateway = GATEWAY.lock().unwrap();
    let order = gateway.create_order(req.amount_usd, &req.chain).unwrap();
    
    // Save to database
    save_order(&order).await;
    
    Json(order)
}

async fn check_order_handler(
    Json(req): Json<CheckOrderRequest>,
) -> Json<PaymentOrder> {
    let order = get_order(&req.order_id).await.unwrap();
    Json(order)
}

#[derive(Deserialize)]
struct CreateOrderRequest {
    amount_usd: f64,
    chain: String,
}

#[derive(Deserialize)]
struct CheckOrderRequest {
    order_id: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/create-order", post(create_order_handler))
        .route("/check-order", post(check_order_handler));
    
    // Start payment monitor in background
    tokio::spawn(async {
        let gateway = GATEWAY.lock().unwrap();
        let mut orders = get_pending_orders().await;
        gateway.monitor_payments(&mut orders).await;
    });
    
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

---

## Step 6: Frontend Integration

```javascript
// Create payment
const response = await fetch('/create-order', {
    method: 'POST',
    body: JSON.stringify({ amount_usd: 99.99, chain: 'BTC' })
});
const order = await response.json();

// Display to customer
document.getElementById('payment-address').textContent = order.payment_address;
document.getElementById('amount').textContent = `${order.expected_amount} satoshis`;

// Poll for confirmation
const checkPayment = setInterval(async () => {
    const status = await fetch('/check-order', {
        method: 'POST',
        body: JSON.stringify({ order_id: order.id })
    }).then(r => r.json());
    
    if (status.status === 'Confirmed') {
        clearInterval(checkPayment);
        showSuccess();
    }
}, 10000);
```

---

## Production Considerations

1. **Price Updates**: Fetch real-time prices from exchange APIs
2. **Address Reuse**: Never reuse addresses - derive new one per order
3. **Hot/Cold Wallets**: Sweep payments to cold storage regularly
4. **Webhooks**: Notify your backend on payment confirmation
5. **Refunds**: Implement refund flow for overpayments

---

## Next Steps

- Add more chains: [Solana](../chains/solana.md), [Polygon](../chains/polygon.md)
- Implement [webhooks](./webhooks.md)
- Add [stablecoin support](./stablecoins.md)
