use helius::error::Result;
use retrieval::Retrieval;
use solana_sdk::pubkey::Pubkey;

mod retrieval;
mod types;

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: Create a RESTful API layer
    // TODO: Fix all unwraps
    // TODO: Add update logic as pooling in time
    // TODO: add unit and integration tests
    // TODO: Data Storage (optional)

    let mut cli_args = std::env::args();
    let _args = cli_args.next();
    let cli_account_arg = cli_args.next().unwrap_or_default();
    let account_pubkey = cli_account_arg.as_str().parse::<Pubkey>().unwrap();

    let mut retrieval = Retrieval::new();

    // Load data to memory
    retrieval.load_data(account_pubkey).await.unwrap();

    let sols = retrieval
        .get_account_balance_sol(account_pubkey.to_string())
        .await;
    println!("current account balance: {:?}", sols);

    let binding = retrieval.database.data.as_ref().unwrap();
    let some_random_tx_hash = binding
        .get(&account_pubkey.to_string())
        .unwrap()
        .transactions
        .as_ref()
        .unwrap()
        .values()
        .next()
        .unwrap();
    println!(
        "some_random_tx_hash.signature: {:?}",
        some_random_tx_hash.signature
    );

    let tx = retrieval
        .get_transaction(
            account_pubkey.to_string(),
            some_random_tx_hash.signature.clone(),
        )
        .await
        .unwrap();
    println!("Full transaction: {:?}", tx);

    Ok(())
}
