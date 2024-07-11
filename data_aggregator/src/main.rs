use futures::future::join_all;
use server::{server_log, server_monitor, run_server};
use solana_sdk::pubkey::Pubkey;
use tokio::task::{self};
use types::{DataAggregator, Retrieval};

mod retrieval;
mod server;
mod types;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // TODO: Add update logic as pooling in time

    // TODO: Comment code and refactor a bit
    // TODO: Write readme
    // TODO: Add unit and integration tests

    // TODO: Refactor account_pubkey approach and adapt to crawling by block, the same as blockchain indexers work.
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

    let log_handle = task::spawn(server_log(aggregator.clone(), account.clone(), 3));
    tasks.push(log_handle);

    let monitor_handle = task::spawn(server_monitor(aggregator.clone(), account, 6));
    tasks.push(monitor_handle);

    let (_close_tx, close_rx) = tokio::sync::oneshot::channel();

    // TODO: It could be handled with tasks vector and join_all(tasks), but there is some type problem.
    run_server(aggregator.clone(), close_rx).await?;

    // Join all aggregator background tasks
    let results = join_all(tasks).await;

    // Handle the results of the tasks
    for result in results {
        result??;
    }

    Ok(())
}
