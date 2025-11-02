use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

use super::DataError;

/// Handler general, serializable objects
/// This is ideal for storing objects for which a relational data model is not the best fit
/// or objects used very rarely, i.e. descriptions of complex assets or global settings which are loaded only once.
/// The ID is a string, which allows for naming object by some convention, e.g. "EUR_cal" for
/// the standard EUR area calendar definition instead of just some arbitrary number.
#[async_trait]
pub trait ObjectHandler {
    // insert, get, update and delete for assets
    async fn store_object<T: Serialize + Sync>(
        &self,
        name: &str,
        object: &T,
    ) -> Result<(), DataError>;
    async fn update_object<T: Serialize + Sync>(
        &self,
        id: &str,
        object: &T,
    ) -> Result<(), DataError>;
    async fn get_object<T: DeserializeOwned>(&self, id: &str) -> Result<T, DataError>;
}
