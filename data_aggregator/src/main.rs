use anyhow::Ok;
use axum::Extension;
use axum::{extract::Path, routing::get, Router};
use futures::future::join_all;
use retrieval::{DataAggregator, Retrieval};
use solana_sdk::pubkey::Pubkey;
use std::time::Duration;
use tokio::task::{self};
use tokio::time::interval;
use tower_http::timeout::TimeoutLayer;

mod retrieval;
mod types;

async fn database_monitor(
    aggregator: DataAggregator,
    account: String,
    interval_in_sec: u64,
) -> Result<(), anyhow::Error> {
    let mut interval = interval(Duration::from_secs(interval_in_sec));

    loop {
        tokio::select! {
                _ = interval.tick() => {
                    let retrieval = aggregator.retrieval.read().await;
                    let sol = retrieval.get_account_balance_sol(account.clone()).await;
                    let txs_count = retrieval.get_account_transactions_count(account.clone()).await;

                    println!("Transactions in DB: {:?} Current Balance: {:?}", txs_count, sol);
            }
        }
    }
}

async fn database_update(
    aggregator: DataAggregator,
    account: String,
    interval_in_sec: u64,
) -> Result<(), anyhow::Error> {
    let mut interval = interval(Duration::from_secs(interval_in_sec));

    loop {
        tokio::select! {
                _ = interval.tick() => {
                    let mut retrieval = aggregator.retrieval.write().await;
                    retrieval.database_update(account.clone()).await.unwrap();

                    println!("Database updated");
            }
        }
    }
}

async fn get_account(Extension(aggregator): Extension<DataAggregator>, Path(id): Path<String>) {
    let retrieval = aggregator.retrieval.read().await;
    println!("retrieval test: {:?}", retrieval.database.data);
    println!("get_account id: {:?}", id);
}

async fn get_transaction(Extension(aggregator): Extension<DataAggregator>, Path(id): Path<String>) {
    let retrieval = aggregator.retrieval.read().await;
    println!("retrieval test: {:?}", retrieval.database.data);
    println!("get_transaction id: {:?}", id);
}

//     aggregator: DataAggregator,
async fn run_server(
    close_rx: tokio::sync::oneshot::Receiver<()>,
) -> Result<(), anyhow::Error> {
    let app = Router::new()
        .route("/", get(|| async { "Pong!" }))
        .route("/account/:id", get(get_account))
        .route("/transaction/:id", get(get_transaction))
        .layer(TimeoutLayer::new(Duration::from_secs(5)));
        // .layer(Extension(aggregator));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            _ = close_rx.await;
        })
        .await
        .unwrap();
    
    println!("Server started!");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // TODO: Add database instance to axum get functions
    // TODO: Comment code and refactor a bit
    // TODO: Write readme
    // TODO: Add update logic as pooling in time
    // TODO: Add unit and integration tests

    // TODO: Add Data Storage (optional). I lost a lot of time on tokio/axum problems related to the cargo version,
    // so I needed to pass data storage layer. Also, I heard that you guys are in a hurry.

    // TODO: Fix all unwraps. Since it's not production ready code I used a lot of unwraps,
    // but they should be replaced with matches and proper error handling

    let mut cli_args = std::env::args();
    let _args = cli_args.next();
    let cli_account_arg = cli_args.next().unwrap_or_default();
    let account_pubkey = cli_account_arg.as_str().parse::<Pubkey>().unwrap();
    let account = account_pubkey.to_string();

    let aggregator = DataAggregator::new(Retrieval::new());

    // Load data from API to memory
    aggregator
        .retrieval
        .write()
        .await
        .load_data(account_pubkey)
        .await
        .unwrap();

    // Aggregator background tasks
    let mut tasks = vec![];

    let monitor_handle = task::spawn(database_monitor(aggregator.clone(), account.clone(), 3));
    tasks.push(monitor_handle);

    let update_handle = task::spawn(database_update(aggregator.clone(), account, 6));
    tasks.push(update_handle);

    println!("Starting server...");

    // TODO: Remove later since it's no needed
    let (_, close_rx) = tokio::sync::oneshot::channel();

    // TODO: This can be fixed. It could be handled with tasks vector and join_all(tasks).
    // run_server(aggregator.clone(), close_rx).await?;
    run_server(close_rx).await?;

    // Join all tasks
    let results = join_all(tasks).await;

    // Handle the results of the tasks
    for result in results {
        result??;
    }

    Ok(())
}
