# Solana Data Aggregator
Solana Data Aggregator serves as a lightweight middleware layer that integrates Solana's RPC Nodes and APIs, such as [Helius](https://www.helius.dev/), with indoor API systems.

## Implementation
I chose to implement the server using the [axum](https://crates.io/crates/axum) framework for creating a RESTful API layer, leveraging my familiarity with it and its proven reliability.

The entry point for the program is the `main()` function, which executes the `run_server()` function. This function launches the [axum](https://crates.io/crates/axum) server and manages relevant background tasks:
- server_log(): Prints the server status every 3 seconds.
- server_monitor(): Updates tracked accounts with SOL balance every 6 seconds.

As a lightweight middleware API layer, our server fetches data from the [Helius API](https://www.helius.dev/) and stores it in a local memory database. The core business logic for that resides in the `impl Retrieval`.

While this approach is straightforward and convenient for now, future enhancements may involve implementing crawling mechanisms, such as fetching transaction data block by block or based on specific criteria, similar to block indexers. Since accepted transactions on Solana are immutable, the `server_monitor` background task focuses on updating account data.


To run the program, execute the following commands in your terminal:
```
cargo run
```
or
```
cargo run --release
```

After execution, the CLI will display the following messages:
```
Starting server at "127.0.0.1:3000"
Accounts updated
DB accounts with balances: Ok({})
DB cache status [Transactions: Ok(0) Accounts: Ok(0)]
```

Once the server is running, you can target three endpoints:

### Server testing endpoint

/
```
Get 127.0.0.1:3000
```

Response
```
"Ping? Pong!"
```

### Account fetching

/account/:account_id
```
Get 127.0.0.1:3000/account/
```

Response
```
{"account_pubkey":[152,27,180,172,64,226,227,125,52,213,203,172,8,187,91,161,148,242,38,146,127,41,121,94,139,92,180,217,130,95,224,203],"lamports":25625559441,"owner":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"executable":false,"rent_epoch":18446744073709551615}
```

### Transaction fetching

/transaction/:tx_signature
```
Get 127.0.0.1:3000/transaction/5XiFRQDYp31KxFQtJqqrjTduTZnGaEWffmv4941D34VsX2GpYavU69bpn1xwWtrcS7fE7D5KuXCjpqjQwLHHeifZ
```

Response
```
{"signature":"5XiFRQDYp31KxFQtJqqrjTduTZnGaEWffmv4941D34VsX2GpYavU69bpn1xwWtrcS7fE7D5KuXCjpqjQwLHHeifZ","timestamp":1720605742,"description":"","fee":5001,"fee_payer":"38tFiQmLwmzUHYiCrYKH4pumqWxpdaYvErUsJbmeSZus","slot":276738369,"native_transfers":[{"amount":2039280,"from_user_account":"71eXHafHQ5mDf4ZeA1FPKsKQFR32TMQsq3wukuwyTSDe","to_user_account":"38tFiQmLwmzUHYiCrYKH4pumqWxpdaYvErUsJbmeSZus"}]}
```

## Postman testing example

First, run the server in the terminal.
![Terminal](./doc/1-terminal.jpg)

Check if the server is running.
![Postman ping](./doc/2-postman-ping.jpg)

Find an active Solana account on the mainnet, for example `3sZA1qjF4GBr1XnvFTbU5HXkxpYKRdf1LRvmXqvyuZiK`, and try to retrieve account data.
![Explorer account](./doc/3-explorer-account.png)

Since there is no account in the memory database, it should be fetched and stored in the memory database.
![Postman fetch account](./doc/4-postman-fetch-account.jpg)

Wait for a while and check the updated balance in SOL; for an active account, the balance should change
![Balance updated](./doc/5-balance-updated.jpg)

Find a Solana transaction on the mainnet, for example `4J3w44KSTsykeSiWPDrceCVN38grcz1ng6TEfRi1DUMeB9hiXETmmEUUjr1tL7KzQTsysxRs6cC1G2TNcWvqJnrE`, and fetch it using Postman.
![Explorer transaction](./doc/6-explorer-transaction.png)
![Postman fetch transaction](./doc/7-transaction-fetch.png)

You can also check the DB cache status.
![Postman fetch transaction](./doc/8-transaction-account-status.jpg)

## Tests
The `data_aggregator_tests` require an internet connection to fetch data from the Helius API. To run the tests, simply execute:

```
cargo test
```

## Development
There are still some TODOs in the code for future development.

For formatting and syntax checks, use:
```
cargo +nightly fmt --all
cargo clippy --all -- -D warnings
cargo clippy --tests -- -D warnings
```

PRs are welcome 😄