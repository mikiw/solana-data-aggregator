use helius::error::Result;
use helius::types::*;
use helius::Helius;
use solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use solana_sdk::pubkey::Pubkey;

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: add mapping from encoded tx
    // TODO: add crawler in time logic
    // TODO: handle unwraps
    // TODO: handle cli arg

    let api_key: &str = "24cf0798-4008-4c81-aa5e-2875323278cd";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    // Replace the following public key with your account's public key
    let account_pubkey_str = "4LLVkfZMp5hh1fEKVexyMNSLjpJyQswvXMW76DaTpyqJ";
    let account_pubkey = account_pubkey_str.parse::<Pubkey>().unwrap();

    let account = helius.rpc().solana_client.get_account(&account_pubkey);
    println!("account: {:?}", account);

    let signature_config = GetConfirmedSignaturesForAddress2Config {
        before: None,
        until: None,
        limit: Some(2), // Limit to the last 10 transactions
        commitment: None,
    };

    let signatures = helius
        .rpc()
        .solana_client
        .get_signatures_for_address_with_config(&account_pubkey, signature_config);

    for signature_info in signatures.unwrap() {
        println!("signature_info block_time: {:?}", signature_info.block_time);
        println!("signature_info slot: {:?}", signature_info.slot);
        println!("signature_info signature: {:?}", signature_info.signature);
    
        let request: ParseTransactionsRequest = ParseTransactionsRequest {
            transactions: vec![
                signature_info.signature,
            ],
        };
    
        let response = helius.parse_transactions(request).await;
        let txs = response.unwrap();
        for tx in txs {
            println!("signature: {:?}", tx.signature);

            println!("description: {:?}", tx.description);
            println!("transaction_type: {:?}", tx.transaction_type);
            println!("transaction_error: {:?}", tx.transaction_error);
            println!("fee: {:?}", tx.fee);
            println!("source: {:?}", tx.source);
            println!("slot: {:?}", tx.slot);
            println!("timestamp: {:?}", tx.timestamp);
 
            for native_transfer in tx.native_transfers.unwrap() {
                println!("from_user_account: {:?}", native_transfer.user_accounts.from_user_account);
                println!("to_user_account: {:?}", native_transfer.user_accounts.to_user_account);
                println!("amount: {:?}", native_transfer.amount);
            }
        }
    }

    Ok(())
}
