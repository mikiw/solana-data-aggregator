use solana_sdk::{pubkey::Pubkey, signature::Signature};
use solana_client::{rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient}, rpc_config::RpcTransactionConfig};
use solana_transaction_status::UiTransactionEncoding;

fn main() {
    // URL to Solana RPC
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());

    // Replace the following public key with your account's public key
    let account_pubkey_str = "4tBxVjsG4pUTnust1MKYzSj7cLcdVP3HPpR7cCdxFX6d";
    let account_pubkey = account_pubkey_str.parse::<Pubkey>().unwrap();

    // Fetch account data
    match client.get_account(&account_pubkey) {
        Ok(account) => {
            // Display account data
            println!("Account data: {:?}", account);

            // Convert lamports to SOL (1 SOL = 1_000_000_000 lamports)
            let sol_balance = account.lamports as f64 / 1_000_000_000.0;
            println!("Balance: {} SOL", sol_balance);
        },
        Err(err) => {
            // Display error message if fetching account data fails
            eprintln!("Error fetching account data: {:?}", err);
        }
    }

    // TODO: add mapping from encoded tx
    // TODO: add crawler in time logic

    // Define the configuration for fetching transaction signatures
    let signature_config = GetConfirmedSignaturesForAddress2Config {
        before: None,
        until: None,
        limit: Some(2), // Limit to the last 10 transactions
        commitment: None,
    };

    // Fetch transaction signatures
    match client.get_signatures_for_address_with_config(&account_pubkey, signature_config) {
        Ok(signatures) => {
            println!("Fetch transaction signatures");

            for signature_info in signatures {
                println!("signature_info block_time: {:?}", signature_info.block_time);
                println!("signature_info slot: {:?}", signature_info.slot);
                println!("signature_info signature: {:?}", signature_info.signature);

                // Convert String to Signature
                let signature = signature_info.signature.parse::<Signature>().unwrap();

                // Fetch transaction details for each signature
                match client.get_transaction_with_config(&signature, RpcTransactionConfig {
                    encoding: Some(UiTransactionEncoding::Json),
                    commitment: None,
                    max_supported_transaction_version: Some(0),
                }) {
                    Ok(transaction) => {
                        println!("Transaction block_time: {:?}", transaction.block_time);
                        println!("Transaction transaction: {:?}", transaction.transaction);
                        // println!("Transaction transaction.transaction: {:?}", transaction.transaction.transaction);
                    },
                    Err(err) => {
                        eprintln!("Error fetching transaction details for signature {}: {:?}", signature_info.signature, err);
                    }
                }
                
                // // Fetch transaction details for each signature
                // match client.get_transaction(&signature_info.signature, RpcTransactionConfig { encoding: None, commitment: None }) {
                //     Ok(transaction) => {
                //         println!("Transaction details: {:?}", transaction);
                //     },
                //     Err(err) => {
                //         eprintln!("Error fetching transaction details for signature {}: {:?}", signature_info.signature, err);
                //     }
                // }
            }
        },
        Err(err) => {
            eprintln!("Error fetching transaction signatures: {:?}", err);
        }
    }
}