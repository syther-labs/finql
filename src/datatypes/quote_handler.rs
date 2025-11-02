//! Data handler trait for market quotes
use async_trait::async_trait;
use std::sync::Arc;
use time::OffsetDateTime;

use super::AssetHandler;
use super::DataError;
use super::{Currency, CurrencyISOCode, Quote, Ticker};

/// Handler for globally available market quotes data
#[async_trait]
pub trait QuoteHandler: AssetHandler {
    fn into_arc_dispatch(self: Arc<Self>) -> Arc<dyn AssetHandler + Send + Sync>;

    // insert, get, update and delete for market data sources
    async fn insert_ticker(&self, ticker: &Ticker) -> Result<i32, DataError>;
    async fn get_ticker_id(&self, ticker: &str) -> Option<i32>;
    async fn insert_if_new_ticker(&self, ticker: &Ticker) -> Result<i32, DataError>;
    async fn get_ticker_by_id(&self, id: i32) -> Result<Ticker, DataError>;
    async fn get_all_ticker(&self) -> Result<Vec<Ticker>, DataError>;
    async fn get_all_ticker_for_source(&self, source: &str) -> Result<Vec<Ticker>, DataError>;

    /// Get all ticker that belong to a given asset specified by its asset ID
    async fn get_all_ticker_for_asset(&self, asset_id: i32) -> Result<Vec<Ticker>, DataError>;

    async fn update_ticker(&self, ticker: &Ticker) -> Result<(), DataError>;
    async fn delete_ticker(&self, id: i32) -> Result<(), DataError>;

    /// Insert, get, update and delete for market data sources
    async fn insert_quote(&self, quote: &Quote) -> Result<i32, DataError>;

    /// Get the last quote in database for a specific currency iso code on or before the given time
    async fn get_last_fx_quote_before(
        &self,
        curr: &CurrencyISOCode,
        time: OffsetDateTime,
    ) -> Result<(Quote, Currency), DataError>;

    /// Get the last quote in database for a specific asset id on or before the given time
    async fn get_last_quote_before_by_id(
        &self,
        asset_id: i32,
        time: OffsetDateTime,
    ) -> Result<(Quote, Currency), DataError>;

    /// Get all quotes within a time range for a specific asset id
    async fn get_quotes_in_range_by_id(
        &self,
        asset_id: i32,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Vec<(Quote, i32)>, DataError>;

    /// Get date of latest quote for a specific ticker id
    async fn get_latest_quote_date_for_ticker(
        &self,
        ticker_id: i32,
    ) -> Result<Option<OffsetDateTime>, DataError>;

    async fn get_all_quotes_for_ticker(&self, ticker_id: i32) -> Result<Vec<Quote>, DataError>;

    /// Update a quote in the database
    async fn update_quote(&self, quote: &Quote) -> Result<(), DataError>;

    /// Delete a quote from the database
    async fn delete_quote(&self, id: i32) -> Result<(), DataError>;

    /// Delete quotes for a specific ticker id within a time range (after start, before or equal to end)
    async fn delete_quotes_for_ticker_id_in_range(
        &self,
        ticker_id: i32,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<(), DataError>;

    /// Remove duplicate quotes from the database
    async fn remove_duplicates(&self) -> Result<(), DataError>;
}
