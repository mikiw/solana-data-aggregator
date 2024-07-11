use anyhow::Error;
use rand::{distributions::Alphanumeric, Rng};
use std::collections::HashMap;
use helius::{
    types::{Cluster, ParseTransactionsRequest},
    Helius,
};
use solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use solana_sdk::pubkey::Pubkey;

use crate::types::{Account, AccountData, Database, Retrieval, Transaction};

impl Retrieval {
    pub fn new() -> Self {
        // Since it's private repo api_key can he here, once it will change for public
        // this api_key needs to be deprecated on Helius page
        let helius = match Helius::new("24cf0798-4008-4c81-aa5e-2875323278cd", Cluster::MainnetBeta)
        {
            Ok(helius) => helius,
            Err(error) => panic!("Cannot establish Helius API connection:: {:?}", error),
        };

        Retrieval {
            helius,
            database: Database { data: None },
        }
    }

    // Function is loading data from API to memory.
    // This might change in future once the database layer is added.
    //
    // Currently the whole workflow of the program is based on account tracking,
    // this is why we need pass account_pubkey.
    //
    // TODO: Later remove account_pubkey and adapt to crawling by block, same as blockchain indexers works.
    pub async fn load_data(&mut self, account_pubkey: Pubkey) -> Result<(), Error> {
        // TODO: Change self.helius.rpc() to async/await
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

        let mut transactions: HashMap<String, Transaction> = HashMap::new();
        
        let response = self.helius.parse_transactions(request).await.unwrap();

        // TODO: change for to map
        for tx in response {
            transactions.insert(
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

        let account = AccountData {
            account: Account {
                account_pubkey,
                owner: account_data.owner,
                lamports: account_data.lamports,
                executable: account_data.executable,
                rent_epoch: account_data.rent_epoch,
            },
            transactions: Some(transactions),
        };

        let mut data: HashMap<String, AccountData> = HashMap::new();
        data.insert(account_pubkey.to_string(), account);

        // Fill memory database with fetched data
        self.set_database(Database { data: Some(data) }).await?;

        Ok(())
    }

    pub async fn set_database(&mut self, db: Database) -> Result<(), Error> {
        self.database = db;
        println!("Database set: {:?}", self.database);

        Ok(())
    }

    pub async fn database_update(&mut self, account: String) -> Result<(), Error> {
        // TODO: implement database update
        
        // TODO: current_block_height will be helpful later
        // let current_block_height = self.helius.rpc().solana_client.get_block_height().unwrap();
        // println!("current_block_height: {:?}", current_block_height);

        // let block = self.helius.rpc()
        //     .solana_client.get_block(current_block_height - 10).unwrap();
        // println!("block: {:?}", block);

        let data = self.database.data.as_mut();
        match data {
            Some(data) => {
                let account = data.get_mut(&account.to_string()).unwrap();
                let dummy_tx = Transaction {
                    description: "test".to_string(),
                    fee: 0,
                    fee_payer: "".to_string(),
                    signature: "".to_string(),
                    timestamp: 0,
                    slot: 0,
                };
                let dummy_random_key: String = rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(7)
                    .map(char::from)
                    .collect();

                account
                    .transactions
                    .as_mut()
                    .unwrap()
                    .insert(dummy_random_key, dummy_tx);

                Ok(())
            }
            _ => Err(Error::msg("Database is not set!")),
        }
    }

    pub async fn get_account_transactions_count(&self, account: String) -> Result<usize, Error> {
        match &self.database.data {
            Some(data) => {
                let account = data.get(&account.to_string()).unwrap();
                let transactions_count = account.transactions.as_ref().unwrap().len();

                Ok(transactions_count)
            }
            _ => Err(Error::msg("Database is not set!")),
        }
    }

    pub async fn get_account_balance_sol(&self, account: String) -> Result<f64, Error> {
        match &self.database.data {
            Some(data) => {
                let account = data.get(&account.to_string()).unwrap();
                let sol_balance = account.account.lamports as f64 / 1_000_000_000.0;

                Ok(sol_balance)
            }
            _ => Err(Error::msg("Database is not set!")),
        }
    }

    pub async fn get_account(&self, account: String) -> Result<Account, Error> {
        match &self.database.data {
            Some(data) => {
                let account = data.get(&account.to_string()).unwrap();

                Ok(account.account.clone())
            }
            _ => Err(Error::msg("Database is not set!")),
        }
    }

    // TODO: Later remove account_pubkey and adapt to crawling by block, same as blockchain indexers works.
    pub async fn get_transaction(
        &self,
        account_pubkey: String,
        tx_hash: String,
    ) -> Result<Transaction, Error> {
        let account = self
            .database
            .data
            .as_ref()
            .unwrap()
            .get(&account_pubkey.to_string())
            .unwrap();

        let transaction = account
            .transactions
            .as_ref()
            .unwrap()
            .get(&tx_hash)
            .unwrap();

        Ok(transaction.clone())
    }
}
