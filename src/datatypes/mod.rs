//! Implementation of a data handler trait to deal with global data
use serde_json;
use sqlx;
use thiserror::Error;

pub mod asset;
pub mod asset_handler;
pub mod cash_flow;
pub mod currency;
pub mod date_time_helper;
pub mod object_handler;
pub mod quote;
pub mod quote_handler;
pub mod stock;
pub mod transaction;
pub mod transaction_handler;

pub use asset::{Asset, AssetSelector};
pub use asset_handler::AssetHandler;
pub use cash_flow::{CashAmount, CashFlow};
pub use currency::{Currency, CurrencyConverter, CurrencyError, CurrencyISOCode};
pub use object_handler::ObjectHandler;
pub use quote::{Quote, Ticker};
pub use quote_handler::QuoteHandler;
pub use stock::Stock;
pub use transaction::{Transaction, TransactionType};
pub use transaction_handler::TransactionHandler;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Database transaction error")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Object (de)serialization error")]
    SerializeError(#[from] serde_json::Error),
    #[error("Connection to database failed: {0}")]
    DataAccessFailure(String),
    #[error("could not found request object in database: {0}")]
    NotFound(String),
    #[error("invalid asset data: {0}")]
    InvalidAsset(String),
    #[error("invalid transaction type: {0}")]
    InvalidTransaction(String),
    #[error("Invalid currency")]
    InvalidCurrency(#[from] CurrencyError),
    #[error("Indetermined time zone offset")]
    InvalidDateTime(#[from] time::error::IndeterminateOffset),
}

pub trait DataItem {
    // get id or return error if id hasn't been set yet
    fn get_id(&self) -> Result<i32, DataError>;
    // set id or return error if id has already been set
    fn set_id(&mut self, id: i32) -> Result<(), DataError>;
}
