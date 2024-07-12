# Solana Data Aggregator
Solana Data Aggregator is a lightweight middleware layer that integrates Solana's RPC Nodes and APIs like [Helius](https://www.helius.dev/) with indoor APIs systems.

## Implementation
I decided to implement [axum](https://crates.io/crates/axum) server as a RESTful API layer since I'm already familiar with that and I think that it's the most tested cargo for that purpose.

Entrypoint for program is `main()` function, that is executing `run_server()` function, that it's running [axum](https://crates.io/crates/axum) server and also relevant background tasks like `server_log()` (that prints server status once every 3 second) and `server_monitor()` (that updates tracked accounts with SOL balance once every 6 seconds).

As a lightweight middleware API layer our server is fetching data from [Helius API](https://www.helius.dev/) and store it in local memory database. Responsible business logic is in `impl Retrieval`.

This approach is easy and convenient for now but in future, some crawling mechanisms like fetching transaction data block by block or with some criteria can be implemented (similar to block indexers). Since accepted transactions on Solana are immutable only account data can be updated with the `server_monitor` background task.

To run program execute:
```
cargo run
```
or
```
cargo run --release
```

CLI after execution:
```
Starting server at "127.0.0.1:3000"
Accounts updated
DB accounts with balances: Ok({})
DB cache status [Transactions: Ok(0) Accounts: Ok(0)]
```

Once server is running you can target 3 endpoints:

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

Firstly run server in terminal.
![Terminal](./doc/1-terminal.jpg)

Check if server is running.
![Postman ping](./doc/2-postman-ping.jpg)

Find active Solana account on mainnet for example `3sZA1qjF4GBr1XnvFTbU5HXkxpYKRdf1LRvmXqvyuZiK` and try to get account data.
![Explorer account](./doc/3-explorer-account.png)

Since there is no account in memory database it should be fetched and stored in memory database.
![Postman fetch account](./doc/4-postman-fetch-account.jpg)

Wait a while and check updated balance in SOL, for active account balance should change.
![Balance updated](./doc/5-balance-updated.jpg)

Find Solana transaction on mainnet for example `4J3w44KSTsykeSiWPDrceCVN38grcz1ng6TEfRi1DUMeB9hiXETmmEUUjr1tL7KzQTsysxRs6cC1G2TNcWvqJnrE` and fetch it in postman.
![Explorer transaction](./doc/6-explorer-transaction.png)
![Postman fetch transaction](./doc/7-transaction-fetch.png)

You can also check DB cache status
![Postman fetch transaction](./doc/8-transaction-account-status.jpg)

## Tests
`data_aggregator_tests` tests requires internet connection to fetch data from Helius API, to run test simply execute:
```
cargo test
```

## Development
There are still some TODOs in the code for future development.

For formatting and syntax checks use:
```
cargo +nightly fmt --all
cargo clippy --all -- -D warnings
cargo clippy --tests -- -D warnings
```

PRs are welcome 😄