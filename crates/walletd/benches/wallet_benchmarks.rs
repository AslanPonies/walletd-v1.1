//! WalletD Benchmark Suite
//!
//! Run benchmarks with:
//! ```bash
//! cargo bench --features "bitcoin ethereum monero"
//! ```

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

// ============================================================================
// Amount Benchmarks
// ============================================================================

fn bench_amount_operations(c: &mut Criterion) {
    use walletd_traits::Amount;

    let mut group = c.benchmark_group("Amount Operations");

    group.bench_function("from_human_18_decimals", |b| {
        b.iter(|| Amount::from_human(black_box(1.5), black_box(18)))
    });

    group.bench_function("from_human_8_decimals", |b| {
        b.iter(|| Amount::from_human(black_box(1.5), black_box(8)))
    });

    group.bench_function("from_smallest_unit", |b| {
        b.iter(|| Amount::from_smallest_unit(black_box(1_000_000_000_000_000_000), black_box(18)))
    });

    group.bench_function("human_readable", |b| {
        let amount = Amount::from_smallest_unit(1_500_000_000_000_000_000, 18);
        b.iter(|| black_box(&amount).human_readable())
    });

    group.bench_function("display_format", |b| {
        let amount = Amount::from_human(1.5, 18);
        b.iter(|| format!("{}", black_box(&amount)))
    });

    group.finish();
}

// ============================================================================
// Ethereum Amount Benchmarks
// ============================================================================

#[cfg(feature = "ethereum")]
fn bench_ethereum_amount(c: &mut Criterion) {
    use walletd_ethereum::EthereumAmount;
    use alloy::primitives::U256;

    let mut group = c.benchmark_group("Ethereum Amount");

    group.bench_function("from_eth", |b| {
        b.iter(|| EthereumAmount::from_eth(black_box(1.5)))
    });

    group.bench_function("from_wei", |b| {
        let wei = U256::from(1_500_000_000_000_000_000u128);
        b.iter(|| EthereumAmount::from_wei(black_box(wei)))
    });

    group.bench_function("to_eth", |b| {
        let amount = EthereumAmount::from_eth(1.5);
        b.iter(|| black_box(&amount).eth())
    });

    group.bench_function("to_gwei", |b| {
        let amount = EthereumAmount::from_eth(1.5);
        b.iter(|| black_box(&amount).gwei())
    });

    group.finish();
}

// ============================================================================
// Monero Amount Benchmarks
// ============================================================================

#[cfg(feature = "monero")]
fn bench_monero_amount(c: &mut Criterion) {
    use walletd_monero::MoneroAmount;

    let mut group = c.benchmark_group("Monero Amount");

    group.bench_function("from_xmr", |b| {
        b.iter(|| MoneroAmount::from_xmr(black_box(1.5)))
    });

    group.bench_function("from_piconero", |b| {
        b.iter(|| MoneroAmount::from_piconero(black_box(1_500_000_000_000)))
    });

    group.bench_function("as_xmr", |b| {
        let amount = MoneroAmount::from_xmr(1.5);
        b.iter(|| black_box(&amount).as_XMR())
    });

    group.bench_function("to_bytes", |b| {
        let amount = MoneroAmount::from_xmr(1.5);
        b.iter(|| black_box(&amount).to_bytes())
    });

    group.bench_function("arithmetic_add", |b| {
        let a = MoneroAmount::from_xmr(1.0);
        let b_amt = MoneroAmount::from_xmr(2.0);
        b.iter(|| black_box(a) + black_box(b_amt))
    });

    group.finish();
}

// ============================================================================
// Bitcoin Wallet Benchmarks
// ============================================================================

#[cfg(feature = "bitcoin")]
fn bench_bitcoin_wallet(c: &mut Criterion) {
    use walletd_bitcoin::{BitcoinWallet, BitcoinWalletBuilder};
    use bdk::keys::bip39::Mnemonic;

    let mnemonic = Mnemonic::parse("outer ride neither foil glue number place usage ball shed dry point").unwrap();

    let mut group = c.benchmark_group("Bitcoin Wallet");

    group.bench_function("builder_new", |b| {
        b.iter(|| BitcoinWalletBuilder::new())
    });

    group.bench_function("wallet_build", |b| {
        let mnemonic = mnemonic.clone();
        b.iter(|| {
            BitcoinWallet::builder()
                .mnemonic(black_box(mnemonic.clone()))
                .build()
                .unwrap()
        })
    });

    group.bench_function("receive_address", |b| {
        let mnemonic = mnemonic.clone();
        let wallet = BitcoinWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();
        b.iter(|| black_box(&wallet).receive_address())
    });

    group.finish();
}

// ============================================================================
// Address Generation Benchmarks
// ============================================================================

#[cfg(feature = "monero")]
fn bench_monero_address(c: &mut Criterion) {
    use walletd_monero::{MoneroPrivateKeys, MoneroPublicKeys, Address, AddressType, Network};

    let seed = [0x66u8; 32]; // Test seed
    let private_keys = MoneroPrivateKeys::from_seed(&seed).unwrap();
    let public_keys = MoneroPublicKeys::from_private_keys(&private_keys);

    let mut group = c.benchmark_group("Monero Address");

    group.bench_function("keys_from_seed", |b| {
        b.iter(|| MoneroPrivateKeys::from_seed(black_box(&seed)))
    });

    group.bench_function("public_from_private", |b| {
        b.iter(|| MoneroPublicKeys::from_private_keys(black_box(&private_keys)))
    });

    group.bench_function("address_new", |b| {
        b.iter(|| {
            Address::new(
                black_box(&Network::Mainnet),
                black_box(&public_keys),
                black_box(&AddressType::Standard),
            )
        })
    });

    group.bench_function("address_to_string", |b| {
        let address = Address::new(&Network::Mainnet, &public_keys, &AddressType::Standard).unwrap();
        b.iter(|| format!("{}", black_box(&address)))
    });

    group.finish();
}

// ============================================================================
// Cryptographic Benchmarks
// ============================================================================

fn bench_crypto_operations(c: &mut Criterion) {
    use walletd_core::{ct_eq, ct_eq_32};

    let mut group = c.benchmark_group("Crypto Operations");

    // Constant-time comparison
    let data_a = [0u8; 32];
    let data_b = [0u8; 32];
    let data_c = [1u8; 32];

    group.bench_function("ct_eq_32_equal", |b| {
        b.iter(|| ct_eq_32(black_box(&data_a), black_box(&data_b)))
    });

    group.bench_function("ct_eq_32_different", |b| {
        b.iter(|| ct_eq_32(black_box(&data_a), black_box(&data_c)))
    });

    group.bench_function("ct_eq_slice_32", |b| {
        b.iter(|| ct_eq(black_box(&data_a[..]), black_box(&data_b[..])))
    });

    // Variable length comparison
    for size in [16, 32, 64, 128, 256].iter() {
        let a = vec![0u8; *size];
        let b = vec![0u8; *size];
        
        group.bench_with_input(BenchmarkId::new("ct_eq_bytes", size), size, |bench, _| {
            bench.iter(|| ct_eq(black_box(&a), black_box(&b)))
        });
    }

    group.finish();
}

// ============================================================================
// Serialization Benchmarks
// ============================================================================

fn bench_serialization(c: &mut Criterion) {
    use walletd_traits::{Amount, TxHash, Network};

    let mut group = c.benchmark_group("Serialization");

    let amount = Amount::from_human(1.5, 18);
    let tx_hash = TxHash::new("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
    let network = Network::mainnet("ethereum").with_chain_id(1);

    group.bench_function("amount_to_json", |b| {
        b.iter(|| serde_json::to_string(black_box(&amount)))
    });

    group.bench_function("amount_from_json", |b| {
        let json = serde_json::to_string(&amount).unwrap();
        b.iter(|| serde_json::from_str::<Amount>(black_box(&json)))
    });

    group.bench_function("tx_hash_to_json", |b| {
        b.iter(|| serde_json::to_string(black_box(&tx_hash)))
    });

    group.bench_function("network_to_json", |b| {
        b.iter(|| serde_json::to_string(black_box(&network)))
    });

    group.finish();
}

// ============================================================================
// ICP Benchmarks
// ============================================================================

#[cfg(feature = "icp")]
fn bench_icp_operations(c: &mut Criterion) {
    use walletd_icp::{IcpWallet, Principal, HDNetworkType};

    let mut group = c.benchmark_group("ICP Operations");

    let principal = Principal::anonymous();

    group.bench_function("principal_to_account_id", |b| {
        b.iter(|| IcpWallet::principal_to_account_id(black_box(&principal)))
    });

    group.bench_function("wallet_from_principal", |b| {
        b.iter(|| IcpWallet::from_principal(black_box(principal), black_box(HDNetworkType::MainNet)))
    });

    group.finish();
}

// ============================================================================
// Criterion Configuration
// ============================================================================

// Base benchmarks (always run)
criterion_group!(
    base_benches,
    bench_amount_operations,
    bench_crypto_operations,
    bench_serialization,
);

// Feature-gated benchmark groups
#[cfg(feature = "ethereum")]
criterion_group!(ethereum_benches, bench_ethereum_amount);

#[cfg(feature = "monero")]
criterion_group!(monero_benches, bench_monero_amount, bench_monero_address);

#[cfg(feature = "bitcoin")]
criterion_group!(bitcoin_benches, bench_bitcoin_wallet);

#[cfg(feature = "icp")]
criterion_group!(icp_benches, bench_icp_operations);

// Main entry point - conditionally include groups
criterion_main!(
    base_benches,
    #[cfg(feature = "ethereum")]
    ethereum_benches,
    #[cfg(feature = "monero")]
    monero_benches,
    #[cfg(feature = "bitcoin")]
    bitcoin_benches,
    #[cfg(feature = "icp")]
    icp_benches,
);
