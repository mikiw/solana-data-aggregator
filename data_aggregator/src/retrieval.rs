use anyhow::Error;
use helius::{
    types::{Cluster, ParseTransactionsRequest},
    Helius,
};
use indexmap::IndexMap;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

use crate::types::{Account, Database, NativeTransfer, Retrieval, Transaction};

impl Retrieval {
    pub fn new() -> Self {
        // Since this is a private repo, the api_key can be included here. Once it becomes public,
        // this api_key needs to be deprecated on the Helius page.
        let helius = match Helius::new("24cf0798-4008-4c81-aa5e-2875323278cd", Cluster::MainnetBeta)
        {
            Ok(helius) => helius,
            Err(error) => panic!("Cannot establish Helius API connection: {:?}", error),
        };

        Retrieval {
            helius,
            database: Database {
                accounts: HashMap::new(),
                transactions: HashMap::new(),
            },
        }
    }

    pub async fn get_account_balances(&self) -> Result<IndexMap<String, f64>, Error> {
        // IndexMap is here to persist order of elements in logs
        let balances: IndexMap<String, f64> = self
            .database
            .accounts
            .iter()
            .map(|(_, account)| {
                (
                    account.account_pubkey.to_string().clone(),
                    account.lamports as f64 / 1_000_000_000.0,
                )
            })
            .collect();

        Ok(balances)
    }

    pub async fn get_account_count(&self) -> Result<usize, Error> {
        Ok(self.database.accounts.len())
    }

    pub async fn update_accounts(&mut self) -> Result<(), Error> {
        let account_keys: Vec<String> = self.database.accounts.keys().cloned().collect();

        // TODO: This implementation is based on naive assumptions and is suitable only for a small number of accounts.
        // For production, implement a robust querying logic (get_multiple_accounts() can be used).)
        for account_id in account_keys
        {
            self.fetch_account(account_id).await?;
        }

        Ok(())
    }

    pub async fn fetch_account(&mut self, account_id: String) -> Result<Account, Error> {
        let account_pubkey = account_id.as_str().parse::<Pubkey>().unwrap();
        // TODO: replace helius.rpc().solana_client with async/await function
        let account_data = self
            .helius
            .rpc()
            .solana_client
            .get_account(&account_pubkey)
            .unwrap();

        let updated_account = Account {
            account_pubkey,
            owner: account_data.owner,
            lamports: account_data.lamports,
            executable: account_data.executable,
            rent_epoch: account_data.rent_epoch,
        };

        self.database
            .accounts
            .insert(account_id, updated_account.clone());

        Ok(updated_account)
    }

    pub async fn get_account(&self, account_id: String) -> Result<Account, Error> {
        Ok(self
            .database
            .accounts
            .get(&account_id.to_string())
            .unwrap()
            .clone())
    }

    pub async fn account_exists(&self, account_id: String) -> Result<bool, Error> {
        Ok(self.database.accounts.contains_key(&account_id))
    }

    pub async fn get_transaction_count(&self) -> Result<usize, Error> {
        Ok(self.database.transactions.len())
    }

    pub async fn fetch_transaction(&mut self, tx_signature: String) -> Result<Transaction, Error> {
        let request: ParseTransactionsRequest = ParseTransactionsRequest {
            transactions: vec![tx_signature],
        };
        let response = &self.helius.parse_transactions(request).await.unwrap()[0];

        let native_transfers = response
            .native_transfers
            .as_ref()
            .unwrap()
            .into_iter()
            .map(|native_transfer| NativeTransfer {
                amount: native_transfer.amount.as_u64().unwrap(),
                from_user_account: native_transfer.user_accounts.from_user_account.clone(),
                to_user_account: native_transfer.user_accounts.to_user_account.clone(),
            })
            .collect();

        let transaction = Transaction {
            signature: response.signature.clone(),
            timestamp: response.timestamp,
            description: response.description.clone(),
            fee: response.fee,
            fee_payer: response.fee_payer.clone(),
            slot: response.slot,
            native_transfers: Some(native_transfers),
        };

        self.database
            .transactions
            .insert(response.signature.clone(), transaction.clone());

        Ok(transaction)
    }

    pub async fn get_transaction(&self, tx_signature: String) -> Result<Transaction, Error> {
        Ok(self
            .database
            .transactions
            .get(&tx_signature)
            .unwrap()
            .clone())
    }

    pub async fn transaction_exists(&self, tx_hash: String) -> Result<bool, Error> {
        Ok(self.database.transactions.contains_key(&tx_hash))
    }

    // TODO: Implement update_transactions based on some criteria.
    // This can also be achieved by crawling block by block
    // from a certain block height, similar to normal block indexers.
    // pub async fn update_transactions(&mut self) -> Result<(), Error> {
    //     Ok(())
    // }
}
