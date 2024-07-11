use server::run_server;

mod retrieval;
mod server;
mod types;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // TODO: Comment code and refactor a bit, check again.
    // TODO: Write readme add screens from solana explorer, terminal.
    // TODO: Add unit and integration tests covering the happy path and edge cases.

    // TODO: Add Data Storage (optional).
    // TODO: Fix all unwraps. Since this is not production-ready code, I used a lot of unwraps,
    // but they should be replaced with match statements and proper error propagation and handling.

    run_server().await?;

    Ok(())
}
