use std::str::FromStr;
use std::time::Duration;

use axum::{extract::Path, routing::get, Extension, Json, Router};
use futures::future::join_all;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use tokio::task::{self};
use tokio::time::interval;
use tower_http::timeout::TimeoutLayer;

use crate::types::{Account, AppError, DataAggregator, Retrieval, Transaction};

async fn server_log(aggregator: DataAggregator, interval_in_sec: u64) -> Result<(), anyhow::Error> {
    let mut interval = interval(Duration::from_secs(interval_in_sec));

    loop {
        tokio::select! {
                _ = interval.tick() => {
                    let retrieval = aggregator.retrieval.read().await;

                    let balances = retrieval.get_account_balances().await;
                    println!("DB accounts with balances: {:?}", balances);

                    let accounts = retrieval.get_account_count().await;
                    let transactions = retrieval.get_transaction_count().await;
                    println!("DB cache status [Transactions: {:?} Accounts: {:?}]", transactions, accounts);
            }
        }
    }
}

async fn server_monitor(
    aggregator: DataAggregator,
    interval_in_sec: u64,
) -> Result<(), anyhow::Error> {
    let mut interval = interval(Duration::from_secs(interval_in_sec));

    loop {
        tokio::select! {
                _ = interval.tick() => {
                    aggregator.retrieval.write().await.update_accounts().await?;

                    println!("Accounts updated");
            }
        }
    }
}

async fn get_account(
    Extension(aggregator): Extension<DataAggregator>,
    Path(account_id): Path<String>,
) -> Result<Json<Account>, AppError> {
    // account_id validation
    account_id
        .as_str()
        .parse::<Pubkey>()
        .map_err(|_| AppError::BadRequest("Account validation failed.".into()))?;

    {
        // Check if the account exists in the cache
        let read_lock = aggregator.retrieval.read().await;
        match read_lock.account_exists(account_id.clone()).await {
            Ok(true) => {
                // If it exists, retrieve it from the cache
                return read_lock
                    .get_account(account_id)
                    .await
                    .map(Json)
                    .map_err(|_| {
                        AppError::InternalServerError("Failed to get account from cache.".into())
                    });
            }
            Ok(false) => {
                // Proceed to fetch the account from the external source later
            }
            Err(_) => {
                return Err(AppError::InternalServerError(
                    "Account existence check failed.".into(),
                ))
            }
        }
    }

    // Acquire a write lock to fetch and store the account
    let mut write_lock = aggregator.retrieval.write().await;
    write_lock
        .fetch_account(account_id)
        .await
        .map(Json)
        .map_err(|_| AppError::InternalServerError("Failed to fetch account.".into()))
}

async fn get_transaction(
    Extension(aggregator): Extension<DataAggregator>,
    Path(tx_signature): axum::extract::Path<String>,
) -> Result<Json<Transaction>, AppError> {
    // tx_signature validation
    Signature::from_str(&tx_signature)
        .map_err(|_| AppError::BadRequest("Invalid transaction signature format.".into()))?;

    {
        // Check if the transaction exists in the cache
        let read_lock = aggregator.retrieval.read().await;
        match read_lock.transaction_exists(tx_signature.clone()).await {
            Ok(true) => {
                // If it exists, retrieve it from the cache
                return read_lock
                    .get_transaction(tx_signature)
                    .await
                    .map(Json)
                    .map_err(|_| {
                        AppError::InternalServerError("Failed to get transaction from cache.".into())
                    });
            }
            Ok(false) => {
                // Proceed to fetch the transaction from the external source later
            }
            Err(_) => {
                return Err(AppError::InternalServerError(
                    "Transaction existence check failed.".into(),
                ))
            }
        }
    }

    // Acquire a write lock to fetch and store the transaction
    let mut write_lock = aggregator.retrieval.write().await;
    write_lock
        .fetch_transaction(tx_signature)
        .await
        .map(Json)
        .map_err(|_| AppError::InternalServerError("Failed to fetch transaction.".into()))
}

async fn run_axum_serve(
    aggregator: DataAggregator,
    close_rx: tokio::sync::oneshot::Receiver<()>,
) -> Result<(), anyhow::Error> {
    let app = Router::new()
        .route("/", get(|| async { "Ping? Pong!" }))
        .route("/account/:account_id", get(get_account))
        .route("/transaction/:tx_signature", get(get_transaction))
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .layer(Extension(aggregator));

    let address = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("Tcp listener failed.");

    println!("Starting server at {:?}", address);

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            _ = close_rx.await;
        })
        .await
        .expect("Server start failed.");

    Ok(())
}

pub async fn run_server() -> Result<(), anyhow::Error> {
    let aggregator = DataAggregator::new(Retrieval::new());

    // Aggregator background tasks
    let mut tasks = vec![];

    let log_handle = task::spawn(server_log(aggregator.clone(), 3));
    tasks.push(log_handle);

    let monitor_handle = task::spawn(server_monitor(aggregator.clone(), 6));
    tasks.push(monitor_handle);

    let (_close_tx, close_rx) = tokio::sync::oneshot::channel();

    // TODO: This could be handled with a tasks vector and join_all(tasks), but there is a type problem.
    // I'm sure it's fixable since I did something similar a couple of weeks ago.
    run_axum_serve(aggregator.clone(), close_rx).await?;

    // Join all aggregator background tasks
    let results = join_all(tasks).await;

    // Handle the results of the tasks
    for result in results {
        result??;
    }

    Ok(())
}
