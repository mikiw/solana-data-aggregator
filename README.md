# Solana Data Aggregator
Solana Data Aggregator is a lightweight middleware layer that integrates Solana's RPC Nodes and APIs like [Helius](https://www.helius.dev/) with indoor API systems.

## 🌐 Documentation
TODO:

## 🌐 Postman testing examples
TODO:

## 🌐 Tests
`data_aggregator_tests` tests requires internet connection to fetch data from Helius API

To run test simply execute:
```
cargo test
```

## 🌐 Development
There are still some TODOs in the code for future development.

For formatting and syntax checks use:
```
cargo +nightly fmt --all
cargo clippy --all -- -D warnings
cargo clippy --tests -- -D warnings
```
