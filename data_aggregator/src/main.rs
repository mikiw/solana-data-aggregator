use anyhow::Ok;
use axum::{
    extract::{Json, Path},
    http::StatusCode,
    routing::get,
    Router,
};
use tower_http::timeout::TimeoutLayer;

use futures::future::join_all;
use retrieval::{Retrieval, Server};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::{ops::Deref, sync::{Arc, Mutex}};
use std::time::Duration;
use std::{future::IntoFuture, net::SocketAddr};
use tokio::task::{self};
use tokio::time::interval;
use uuid::Uuid;

mod retrieval;
mod types;

async fn print_current_balance_from_db(
    server: Server,
    account: String,
    interval_in_sec: u64,
) -> Result<(), anyhow::Error> {
    let mut interval = interval(Duration::from_secs(interval_in_sec));

    loop {
        tokio::select! {
                _ = interval.tick() => {
                    let retrieval = server.retrieval.read().await;
                    // let txs_count = retrieval.get_account_transactions_count(account.clone()).await;
                    // println!("Transactions count: {:?}", txs_count);

                    let sol = retrieval.get_account_balance_sol(account.clone()).await;
                    println!("Current account balance: {:?}", sol);
            }
        }
    }
}

async fn monitor_data(
    server: Server,
    account: String,
    interval_in_sec: u64,
) -> Result<(), anyhow::Error> {
    let mut interval = interval(Duration::from_secs(interval_in_sec));

    loop {
        tokio::select! {
                _ = interval.tick() => {
                    println!("Monitor_data: {:?}", 0);
            }
        }
    }
}

async fn get_account() {
    println!("get_account");
}

async fn get_transaction() {
    println!("get_transaction");
}

async fn run_server(close_rx: tokio::sync::oneshot::Receiver<()>) -> Result<(), anyhow::Error> {
    let app = Router::new()
    .route("/", get(|| async { "Hello, World!" }))
    .route("/account/", get(get_account))
    .route("/transaction/", get(get_transaction))
    .layer(TimeoutLayer::new(Duration::from_secs(10)));

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            _ = close_rx.await;
        })
        .await
        .unwrap();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // TODO: Create a RESTful API layer
    // TODO: Fix all unwraps
    // TODO: Add update logic as pooling in time
    // TODO: add unit and integration tests
    // TODO: Data Storage (optional)

    let mut cli_args = std::env::args();
    let _args = cli_args.next();
    let cli_account_arg = cli_args.next().unwrap_or_default();
    let account_pubkey = cli_account_arg.as_str().parse::<Pubkey>().unwrap();
    let account = account_pubkey.to_string();

    let server = Server::new(Retrieval::new());

    // Load data to memory
    server.retrieval.write().await.load_data(account_pubkey).await.unwrap();

    // let binding = server_mut.database.data.as_mut().unwrap();
    // let some_random_tx_hash = binding
    //     .get(&account)
    //     .unwrap()
    //     .transactions
    //     .as_ref()
    //     .unwrap()
    //     .values()
    //     .next()
    //     .unwrap();
    // println!(
    //     "some_random_tx_hash.signature: {:?}",
    //     some_random_tx_hash.signature
    // );

    // let tx = server_mut
    //     .get_transaction(
    //         account_pubkey.to_string(),
    //         some_random_tx_hash.signature.clone(),
    //     )
    //     .await
    //     .unwrap();
    // println!("Full transaction: {:?}", tx);

    let mut tasks = vec![];
    
    let balance_handle = task::spawn(print_current_balance_from_db(server.clone(), account.clone(), 5));
    tasks.push(balance_handle);

    let monitor_handle = task::spawn(monitor_data(server.clone(), account, 5));
    tasks.push(monitor_handle);

    print!("start server now...");

    let (close_tx, close_rx) = tokio::sync::oneshot::channel();

    // TODO: fix this later, it should be handled with tasks vector and join_all
    run_server(close_rx).await?;
    
    // join all tasks
    let results = join_all(tasks).await;

    // handle the results of the tasks
    for result in results {
        result??;
    }

    Ok(())
}
