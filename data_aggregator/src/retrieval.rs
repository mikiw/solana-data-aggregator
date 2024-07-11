use std::collections::HashMap;

use helius::{
    types::{Cluster, ParseTransactionsRequest},
    Helius,
};
use solana_client::{
    client_error::reqwest::Error, rpc_client::GetConfirmedSignaturesForAddress2Config,
};
use solana_sdk::pubkey::Pubkey;

use crate::types::{Account, AccountData, Database, Transaction};

pub struct Retrieval {
    pub helius: Helius,
    pub database: Database,
}

impl Retrieval {
    pub fn new() -> Self {
        let helius =
            match Helius::new("24cf0798-4008-4c81-aa5e-2875323278cd", Cluster::MainnetBeta) {
                Ok(helius) => helius,
                Err(error) => panic!("Cannot establish Helius API connection:: {:?}", error),
            };

        Retrieval {
            helius,
            database: Database { data: None },
        }
    }

    pub async fn load_data(&mut self, account_pubkey: Pubkey) -> Result<(), Error> {
        // TODO: comment this load
        // TODO: change account_pubkey to account_pubkeys as Vec...
        // TODO: load_data by blocks?

        // TODO: change self.helius.rpc() to async/await
        let account_data = self
            .helius
            .rpc()
            .solana_client
            .get_account(&account_pubkey)
            .unwrap();

        // TODO: signature_config can be used to crawl data for more than 10 transactions
        let signature_config = GetConfirmedSignaturesForAddress2Config {
            before: None,
            until: None,
            limit: Some(2), // Limit to the last 2 transactions
            commitment: None,
        };

        let signatures = self
            .helius
            .rpc()
            .solana_client
            .get_signatures_for_address_with_config(&account_pubkey, signature_config);

        let request_signatures: Vec<String> = signatures
            .unwrap()
            .iter()
            .map(|tx| tx.signature.clone())
            .collect();
        let request = ParseTransactionsRequest {
            transactions: request_signatures,
        };

        let mut txs: HashMap<String, Transaction> = HashMap::new();
        let response = self.helius.parse_transactions(request).await.unwrap();
        // TODO: change for to map
        for tx in response {
            txs.insert(
                tx.signature.clone(),
                Transaction {
                    signature: tx.signature.clone(),
                    timestamp: tx.timestamp,
                    description: tx.description.clone(),
                    fee: tx.fee,
                    fee_payer: tx.fee_payer.clone(),
                    slot: tx.slot,
                },
            );
        }

        let data = AccountData {
            account: Account {
                account_pubkey,
                owner: account_data.owner,
                lamports: account_data.lamports,
                executable: account_data.executable,
                rent_epoch: account_data.rent_epoch,
            },
            transactions: Some(txs),
        };

        let mut database: HashMap<String, AccountData> = HashMap::new();
        database.insert(account_pubkey.to_string(), data);

        // Fill memory database with fetched data
        self.database.data = Some(database);

        // TODO: current_block_height will be helpful later
        // let current_block_height = self.helius.rpc().solana_client.get_block_height().unwrap();
        // println!("current_block_height: {:?}", current_block_height);

        // let block = self.helius.rpc()
        //     .solana_client.get_block(current_block_height - 10).unwrap();
        // println!("block: {:?}", block);
        Ok(())
    }

    // TODO: implement monitor data in time
    // pub async fn monitor_data(&self, account_pubkey: Pubkey) -> Result<(), Error> {
    //     Ok(())
    // }

    pub async fn get_account_balance_sol(&self, account_pubkey: String) -> Result<f64, Error> {
        // TODO: add error handling and remove unwraps
        let account = self
            .database
            .data
            .as_ref()
            .unwrap()
            .get(&account_pubkey.to_string())
            .unwrap();
        let sol_balance = account.account.lamports as f64 / 1_000_000_000.0;

        Ok(sol_balance)
    }

    // TODO: change this to remove account_pubkey and use only tx hash
    pub async fn get_transaction(
        &self,
        account_pubkey: String,
        tx_hash: String,
    ) -> Result<&Transaction, Error> {
        let account = self
            .database
            .data
            .as_ref()
            .unwrap()
            .get(&account_pubkey.to_string())
            .unwrap();
        let tx = account
            .transactions
            .as_ref()
            .unwrap()
            .get(&tx_hash)
            .unwrap();

        Ok(tx)
    }
}
