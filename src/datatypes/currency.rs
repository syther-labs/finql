use std::fmt;
use std::str::FromStr;

use super::{DataError, DataItem};
use async_trait::async_trait;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::{Eq, PartialEq};
use thiserror::Error;
use time::OffsetDateTime;

/// Error type related to the Currency
#[derive(Error, Debug, PartialEq)]
pub enum CurrencyError {
    #[error("currency codes must consist of exactly three characters")]
    InvalidLength,
    #[error("currency codes must contain only alphabetic ASCII characters")]
    InvalidCharacter,
    #[error("currency deserialization failed")]
    DeserializationFailed,
    #[error("Currency conversion failed")]
    ConversionFailed,
    #[error("Conversion currency not found: {0}")]
    CurrencyNotFound(String),
    #[error("Currency not in database: {0}")]
    CurrencyNotInDatabase(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Missing quote for currency pair {0}/{1}")]
    MissingQuoteForCurrencyPair(String, String),
    #[error("Failed to fetch quote from databasei: {0}")]
    DataBaseError(String),
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct CurrencyISOCode {
    iso_code: [char; 3],
}

impl CurrencyISOCode {
    pub fn new(code: &str) -> Result<CurrencyISOCode, CurrencyError> {
        let mut iso_code = [' ', ' ', ' '];
        let mut idx = 0;
        for c in code.chars() {
            if idx >= 3 {
                return Err(CurrencyError::InvalidLength);
            }

            let c = c.to_ascii_uppercase();
            if c.is_ascii_alphabetic() {
                iso_code[idx] = c.to_ascii_uppercase();
                idx += 1;
            } else {
                return Err(CurrencyError::InvalidCharacter);
            }
        }
        if idx != 3 {
            Err(CurrencyError::InvalidLength)
        } else {
            Ok(Self { iso_code })
        }
    }
}

impl fmt::Display for CurrencyISOCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.iso_code[0], self.iso_code[1], self.iso_code[2]
        )
    }
}

impl FromStr for CurrencyISOCode {
    type Err = CurrencyError;

    fn from_str(c: &str) -> Result<CurrencyISOCode, CurrencyError> {
        Self::new(c)
    }
}

/// Special type for currencies
#[derive(Debug, Clone, Copy)]
pub struct Currency {
    pub id: Option<i32>,
    pub iso_code: CurrencyISOCode,
    pub rounding_digits: i32,
}

impl PartialEq for Currency {
    fn eq(&self, other: &Self) -> bool {
        self.iso_code == other.iso_code
    }
}

impl Eq for Currency {}

impl Currency {
    pub fn new(id: Option<i32>, iso_code: CurrencyISOCode, rounding_digits: Option<i32>) -> Self {
        Self {
            id,
            iso_code,
            rounding_digits: rounding_digits
                .unwrap_or_else(|| default_rounding_digits(&iso_code.to_string())),
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.iso_code,)
    }
}

impl DataItem for Currency {
    fn get_id(&self) -> Result<i32, DataError> {
        match self.id {
            Some(id) => Ok(id),
            None => Err(DataError::DataAccessFailure(
                "Can't get id of temporary currency".to_string(),
            )),
        }
    }

    fn set_id(&mut self, id: i32) -> Result<(), DataError> {
        match self.id {
            Some(_) => Err(DataError::DataAccessFailure(
                "Can't change id of persistent currency".to_string(),
            )),
            None => {
                self.id = Some(id);
                Ok(())
            }
        }
    }
}

fn default_rounding_digits(curr: &str) -> i32 {
    match curr {
        "JPY" | "TRL" => 0,
        _ => 2,
    }
}

/// Transform a string into a Currency
impl FromStr for Currency {
    type Err = CurrencyError;

    fn from_str(c: &str) -> Result<Currency, CurrencyError> {
        Ok(Currency::new(None, CurrencyISOCode::from_str(c)?, None))
    }
}

impl Serialize for Currency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", &self))
    }
}

struct CurrencyVisitor;

impl Visitor<'_> for CurrencyVisitor {
    type Value = Currency;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a currency code must consist of three alphabetic characters")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match Currency::from_str(value) {
            Ok(val) => Ok(val),
            Err(err) => Err(E::custom(format!("{}", err))),
        }
    }
}

impl<'de> Deserialize<'de> for Currency {
    fn deserialize<D>(deserializer: D) -> Result<Currency, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(CurrencyVisitor)
    }
}

impl Currency {
    pub fn rounding_digits(&self) -> i32 {
        self.rounding_digits
    }
}

/// Trait for calculating FX rates for currency conversion
#[async_trait]
pub trait CurrencyConverter {
    /// returns the price of 1 unit of base currency in terms of quote currency
    async fn fx_rate(
        &self,
        base_currency: Currency,
        quote_currency: Currency,
        time: OffsetDateTime,
    ) -> Result<f64, CurrencyError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write_currency() {
        // valid iso code
        let currency = Currency::from_str("EUR").unwrap();
        assert_eq!(format!("{}", currency), "EUR".to_string());

        // case ignorance
        let currency = Currency::from_str("euR").unwrap();
        assert_eq!(format!("{}", currency), "EUR".to_string());

        // to long
        let currency = Currency::from_str("EURO");
        assert_eq!(currency, Err(CurrencyError::InvalidLength));

        // to short
        let currency = Currency::from_str("EU");
        assert_eq!(currency, Err(CurrencyError::InvalidLength));

        // invalid character1
        let currency = Currency::from_str("éUR");
        assert_eq!(currency, Err(CurrencyError::InvalidCharacter));

        // invalid character2
        let currency = Currency::from_str("EU1");
        assert_eq!(currency, Err(CurrencyError::InvalidCharacter));
    }

    #[test]
    fn deserialize_currency() {
        let input = r#""EUR""#;

        let curr: Currency = serde_json::from_str(input).unwrap();
        assert_eq!(format!("{}", curr), "EUR");
    }
    #[test]
    fn serialize_currency() {
        let curr = Currency {
            id: None,
            iso_code: CurrencyISOCode::from_str("EUR").unwrap(),
            rounding_digits: 2,
        };
        let json = serde_json::to_string(&curr).unwrap();
        assert_eq!(json, r#""EUR""#);
    }
}
