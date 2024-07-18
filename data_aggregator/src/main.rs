use server::run_server;

mod retrieval;
mod server;
mod types;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // TODO: Currently, the pooling strategy is implemented with server_monitor.
    // Check the possibility of a push aproach and check features of the Solana rpc,
    // such as listening to the latest processed slot of a validator
    // TODO: Add Persistent Data Storage (optional).
    // TODO: Add more complicated test scenarios in data_aggregator_tests, for example:
    // many transactions in one test, many accounts in one test, tests with fn update_accounts().
    // TODO: Add server integration tests. Probably use the release version and start as the process.
    // TODO: Add load tests with more data.
    // TODO: Check again error handling and propagations with more tests.
    // TODO: Add separation between the data aggregated by the aggregator and the api consuming the data from the aggregator.

    run_server().await?;

    Ok(())
}

// TODO: Move this to newly created test folder
#[cfg(test)]
mod data_aggregator_tests {
    use solana_sdk::pubkey::Pubkey;

    use crate::types::{DataAggregator, Retrieval};

    const ACTIVE_MAINNET_ACCOUNT: &str = "BEmUSjqs7mpgaSXw6QdrePfTsD8aQHbdtnqUxa63La6E";
    const TRANSACTION_WITH_NATIVE_TRANSFERS: &str =
        "5XiFRQDYp31KxFQtJqqrjTduTZnGaEWffmv4941D34VsX2GpYavU69bpn1xwWtrcS7fE7D5KuXCjpqjQwLHHeifZ";
    const USDC_CONTRACT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    const USDC_TRANSACTION: &str =
        "3QGpxQhhDU2ijTnQhBEjYj28judQg7Ymrn5jZxMTmNKXqDSj2jSwNTB7Tfau6tkSF5rA7nT57HPbVeAxF9zpv25b";

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fetch_active_mainnet_account() {
        let aggregator = DataAggregator::new(Retrieval::new());
        let mut retrieval = aggregator.retrieval.write().await;

        let account = retrieval
            .fetch_account(ACTIVE_MAINNET_ACCOUNT.to_string())
            .await
            .unwrap();

        assert_eq!(
            account.account_pubkey,
            ACTIVE_MAINNET_ACCOUNT.parse::<Pubkey>().unwrap()
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn check_if_existing_account_exists() {
        let aggregator = DataAggregator::new(Retrieval::new());
        let mut retrieval = aggregator.retrieval.write().await;

        let exists = retrieval
            .account_exists(ACTIVE_MAINNET_ACCOUNT.to_string())
            .await
            .unwrap();
        assert!(!exists);

        retrieval
            .fetch_account(ACTIVE_MAINNET_ACCOUNT.to_string())
            .await
            .unwrap();

        let exists = retrieval
            .account_exists(ACTIVE_MAINNET_ACCOUNT.to_string())
            .await
            .unwrap();
        assert!(exists);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn get_balances() {
        let aggregator = DataAggregator::new(Retrieval::new());
        let mut retrieval = aggregator.retrieval.write().await;

        let balances = retrieval.get_account_balances().await.unwrap();
        assert_eq!(balances.len(), 0);

        retrieval
            .fetch_account(ACTIVE_MAINNET_ACCOUNT.to_string())
            .await
            .unwrap();

        let balances = retrieval.get_account_balances().await.unwrap();
        assert_eq!(balances.len(), 1);
        assert!(balances.contains_key(ACTIVE_MAINNET_ACCOUNT));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn get_active_mainnet_account() {
        let aggregator = DataAggregator::new(Retrieval::new());
        let mut retrieval = aggregator.retrieval.write().await;

        let account = retrieval
            .get_account(ACTIVE_MAINNET_ACCOUNT.to_string())
            .await;
        assert!(account.is_err());
        let error_message = format!("{}", account.err().unwrap());
        assert_eq!(error_message, "Account not found.");

        retrieval
            .fetch_account(ACTIVE_MAINNET_ACCOUNT.to_string())
            .await
            .unwrap();

        let account = retrieval
            .get_account(ACTIVE_MAINNET_ACCOUNT.to_string())
            .await
            .unwrap();
        assert_eq!(
            account.account_pubkey,
            ACTIVE_MAINNET_ACCOUNT.parse::<Pubkey>().unwrap()
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn get_usdc_mainnet_account() {
        let aggregator = DataAggregator::new(Retrieval::new());
        let mut retrieval = aggregator.retrieval.write().await;

        let account = retrieval.get_account(USDC_CONTRACT.to_string()).await;
        assert!(account.is_err());
        let error_message = format!("{}", account.err().unwrap());
        assert_eq!(error_message, "Account not found.");

        retrieval
            .fetch_account(USDC_CONTRACT.to_string())
            .await
            .unwrap();

        let account = retrieval
            .get_account(USDC_CONTRACT.to_string())
            .await
            .unwrap();
        assert_eq!(
            account.account_pubkey,
            USDC_CONTRACT.parse::<Pubkey>().unwrap()
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn get_account_count() {
        let aggregator = DataAggregator::new(Retrieval::new());
        let mut retrieval = aggregator.retrieval.write().await;

        let account_count = retrieval.get_account_count().await.unwrap();
        assert_eq!(account_count, 0);

        retrieval
            .fetch_account(ACTIVE_MAINNET_ACCOUNT.to_string())
            .await
            .unwrap();

        let account_count = retrieval.get_account_count().await.unwrap();
        assert_eq!(account_count, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn get_transaction_count() {
        let aggregator = DataAggregator::new(Retrieval::new());
        let mut retrieval = aggregator.retrieval.write().await;

        let transactions = retrieval.get_transaction_count().await.unwrap();
        assert_eq!(transactions, 0);

        retrieval
            .fetch_transaction(USDC_TRANSACTION.to_string())
            .await
            .unwrap();

        let transactions = retrieval.get_transaction_count().await.unwrap();
        assert_eq!(transactions, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fetch_usdc_mainnet_transaction() {
        let aggregator = DataAggregator::new(Retrieval::new());
        let mut retrieval = aggregator.retrieval.write().await;

        let transaction = retrieval
            .fetch_transaction(USDC_TRANSACTION.to_string())
            .await
            .unwrap();

        assert_eq!(transaction.signature, USDC_TRANSACTION);
        assert_eq!(transaction.description, "MfDuWeqSHEqTFVYZ7LoexgAK9dxk7cy4DFJWjWMGVWa swapped 4.381112266 SOL for 594.136235 USDC");
        assert_eq!(transaction.timestamp, 1720769593);
        assert_eq!(transaction.fee, 18001);
        assert_eq!(
            transaction.fee_payer,
            "MfDuWeqSHEqTFVYZ7LoexgAK9dxk7cy4DFJWjWMGVWa"
        );
        assert_eq!(transaction.slot, 277087628);
        assert_eq!(transaction.native_transfers.unwrap().len(), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn check_if_existing_transaction_exists() {
        let aggregator = DataAggregator::new(Retrieval::new());
        let mut retrieval = aggregator.retrieval.write().await;

        let exists = retrieval
            .transaction_exists(USDC_TRANSACTION.to_string())
            .await
            .unwrap();
        assert!(!exists);

        retrieval
            .fetch_transaction(USDC_TRANSACTION.to_string())
            .await
            .unwrap();

        let exists = retrieval
            .transaction_exists(USDC_TRANSACTION.to_string())
            .await
            .unwrap();
        assert!(exists);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn get_usdc_mainnet_transaction() {
        let aggregator = DataAggregator::new(Retrieval::new());
        let mut retrieval = aggregator.retrieval.write().await;

        let transaction = retrieval
            .get_transaction(USDC_TRANSACTION.to_string())
            .await;
        assert!(transaction.is_err());
        let error_message = format!("{}", transaction.err().unwrap());
        assert_eq!(error_message, "Transaction not found");

        retrieval
            .fetch_transaction(USDC_TRANSACTION.to_string())
            .await
            .unwrap();

        let transaction = retrieval
            .get_transaction(USDC_TRANSACTION.to_string())
            .await
            .unwrap();

        assert_eq!(transaction.signature, USDC_TRANSACTION);
        assert_eq!(transaction.description, "MfDuWeqSHEqTFVYZ7LoexgAK9dxk7cy4DFJWjWMGVWa swapped 4.381112266 SOL for 594.136235 USDC");
        assert_eq!(transaction.timestamp, 1720769593);
        assert_eq!(transaction.fee, 18001);
        assert_eq!(
            transaction.fee_payer,
            "MfDuWeqSHEqTFVYZ7LoexgAK9dxk7cy4DFJWjWMGVWa"
        );
        assert_eq!(transaction.slot, 277087628);
        assert_eq!(transaction.native_transfers.unwrap().len(), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn get_native_transfers_mainnet_transaction() {
        let aggregator = DataAggregator::new(Retrieval::new());
        let mut retrieval = aggregator.retrieval.write().await;

        let transaction = retrieval
            .get_transaction(TRANSACTION_WITH_NATIVE_TRANSFERS.to_string())
            .await;
        assert!(transaction.is_err());
        let error_message = format!("{}", transaction.err().unwrap());
        assert_eq!(error_message, "Transaction not found");

        retrieval
            .fetch_transaction(TRANSACTION_WITH_NATIVE_TRANSFERS.to_string())
            .await
            .unwrap();

        let transaction = retrieval
            .get_transaction(TRANSACTION_WITH_NATIVE_TRANSFERS.to_string())
            .await
            .unwrap();

        assert_eq!(transaction.signature, TRANSACTION_WITH_NATIVE_TRANSFERS);
        assert_eq!(transaction.description, "");
        assert_eq!(transaction.timestamp, 1720605742);
        assert_eq!(transaction.fee, 5001);
        assert_eq!(
            transaction.fee_payer,
            "38tFiQmLwmzUHYiCrYKH4pumqWxpdaYvErUsJbmeSZus"
        );
        assert_eq!(transaction.slot, 276738369);

        let native_transfers = transaction
            .native_transfers
            .as_ref()
            .expect("Expected some native transfers.");
        assert_eq!(native_transfers.len(), 1);

        let native_transfer = &native_transfers[0];
        assert_eq!(native_transfer.amount, 2039280);
        assert_eq!(
            native_transfer.from_user_account,
            Some("71eXHafHQ5mDf4ZeA1FPKsKQFR32TMQsq3wukuwyTSDe".to_string())
        );
        assert_eq!(
            native_transfer.to_user_account,
            Some("38tFiQmLwmzUHYiCrYKH4pumqWxpdaYvErUsJbmeSZus".to_string())
        );
    }
}
