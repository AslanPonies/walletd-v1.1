# Transaction Broadcasting
```rust
let broadcaster = Broadcaster::new();
let result = broadcaster.broadcast_to(Chain::Bitcoin, &signed_tx).await?;
```
