use std::time::Duration;

use axum::{extract::Path, routing::get, Extension, Json, Router};
use tower_http::timeout::TimeoutLayer;
use tokio::time::interval;

use crate::types::{Account, DataAggregator, Transaction};

pub async fn server_log(
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
                    let transaction_count = retrieval.get_account_transactions_count(account.clone()).await;

                    println!("Transactions in DB: {:?} Current Balance: {:?}", transaction_count, sol);
            }
        }
    }
}

pub async fn server_monitor(
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

async fn get_account(
    Extension(aggregator): Extension<DataAggregator>,
    Path(account_id): Path<String>,
) -> Result<Json<Account>, axum::http::StatusCode> {
    let retrieval = aggregator.retrieval.read().await;

    let account = retrieval
        .get_account(account_id)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(account))
}

async fn get_transaction(
    Extension(aggregator): Extension<DataAggregator>,
    Path((account_id, transaction_id)): axum::extract::Path<(String, String)>,
) -> Result<Json<Transaction>, axum::http::StatusCode> {
    let retrieval = aggregator.retrieval.read().await;

    let transaction = retrieval
        .get_transaction(account_id, transaction_id)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(transaction))
}

pub async fn run_server(
    aggregator: DataAggregator,
    close_rx: tokio::sync::oneshot::Receiver<()>,
) -> Result<(), anyhow::Error> {
    let app = Router::new()
        .route("/", get(|| async { "Pong!" }))
        .route("/account/:account_id", get(get_account))
        .route("/transaction/:account_id/:transaction_id", get(get_transaction))
        .layer(TimeoutLayer::new(Duration::from_secs(5)))
        .layer(Extension(aggregator));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Starting server...");

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            _ = close_rx.await;
        })
        .await
        .unwrap();

    Ok(())
}
