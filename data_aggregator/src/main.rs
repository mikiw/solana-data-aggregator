use futures::future::join_all;
use server::{run_server, server_log, server_monitor};
use tokio::task::{self};
use types::{DataAggregator, Retrieval};

mod retrieval;
mod server;
mod types;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // TODO: Add update logic as pooling in time

    // TODO: Comment code and refactor a bit.
    // TODO: Write readme.
    // TODO: Add unit and integration tests.

    // TODO: Add Data Storage (optional).
    // TODO: Fix all unwraps. Since this is not production-ready code, I used a lot of unwraps,
    // but they should be replaced with match statements and proper error propagation and handling.
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
    run_server(aggregator.clone(), close_rx).await?;

    // Join all aggregator background tasks
    let results = join_all(tasks).await;

    // Handle the results of the tasks
    for result in results {
        result??;
    }

    Ok(())
}
