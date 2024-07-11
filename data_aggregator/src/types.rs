use std::{collections::HashMap, sync::Arc};
use helius::Helius;
use serde::Serialize;
use solana_sdk::pubkey::Pubkey;
use tokio::sync::RwLock;

/// DataAggregator can be shared between threads with read/write lock access
#[derive(Clone)]
pub struct DataAggregator {
    pub retrieval: Arc<RwLock<Retrieval>>,
}

impl DataAggregator {
    pub fn new(retrieval: Retrieval) -> Self {
        Self {
            retrieval: Arc::new(RwLock::new(retrieval)),
        }
    }
}

// TODO: helius and database can be abstracted in future to handle different types of APIs and databases
pub struct Retrieval {
    pub helius: Helius,
    pub database: Database,
}

#[derive(Debug)]
pub struct Database {
    // Account's public key as string is hashmap key to account data
    pub data: Option<HashMap<String, AccountData>>,
}

#[derive(Debug)]
pub struct AccountData {
    pub account: Account,
    // Signature as string is hashmap key to transaction data
    pub transactions: Option<HashMap<String, Transaction>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Account {
    // Account's public key
    pub account_pubkey: Pubkey,
    // Lamports in the account
    pub lamports: u64,
    // The program that owns this account. If executable, the program that loads this account.
    pub owner: Pubkey,
    // This account's data contains a loaded program (and is now read-only)
    pub executable: bool,
    // The epoch at which this account will next owe rent
    pub rent_epoch: u64,
}

// TODO: mapping to everything that is missing. Especially token_transfers,
#[derive(Debug, Clone, Serialize)]
pub struct Transaction {
    pub signature: String,
    pub timestamp: u64,
    pub description: String,
    pub fee: i32,
    pub fee_payer: String,
    pub slot: i32,

    // pub native_transfers: Option<Vec<NativeTransfer>>,

    // // Serializable and deserializable helius transaction data gathered by time of data crawling.
    // // We want to store all available data that can be reusable in the future if needed.
    // pub transaction_data: EnhancedTransaction
}
