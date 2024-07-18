use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use helius::Helius;
use serde::Serialize;
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
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

// TODO: Helius and database can be abstracted in future to handle different
// types of APIs and databases. Generics can be used here.
pub struct Retrieval {
    pub helius: Helius,
    pub database: Database,
}

#[derive(Debug)]
pub struct Database {
    // Account's public key as a string is the hashmap key for account data
    pub accounts: HashMap<String, Account>,
    // The signature as a string serves as the hashmap key for transaction data
    pub transactions: HashMap<String, Transaction>,
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

// TODO: Add mappings to everything from EnhancedTransaction that is missing.
// Especially account_data, instructions, events, token_transfers,
// also read more about account_data and redesign the current code.
#[derive(Debug, Clone, Serialize)]
pub struct Transaction {
    pub signature: String,
    pub timestamp: u64,
    pub description: String,
    pub fee: i32,
    pub fee_payer: String,
    pub slot: i32,
    pub native_transfers: Option<Vec<NativeTransfer>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NativeTransfer {
    pub amount: u64,
    pub from_user_account: Option<String>,
    pub to_user_account: Option<String>,
}

// Define a custom error type
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),
}

// Implement IntoResponse for your custom error type
impl IntoResponse for AppError {
    fn into_response(self) -> Response<Body> {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = format!("{{\"error\": \"{}\"}}", message);
        Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap()
    }
}
