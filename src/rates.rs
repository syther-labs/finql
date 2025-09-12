use serde::{Deserialize, Serialize};
use time::Date;

use crate::datatypes::cash_flow::{CashAmount, CashFlow};
use crate::datatypes::currency::Currency;

use crate::day_count_conv::DayCountConv;

/// Methods for compounding interest rates
#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub enum Compounding {
    #[serde(rename = "simple")]
    Simple,
    #[serde(rename = "annual")]
    Annual,
    #[serde(rename = "semi-annual")]
    SemiAnnual,
    #[serde(rename = "quarterly")]
    Quarterly,
    #[serde(rename = "monthly")]
    Monthly,
    #[serde(rename = "continuous")]
    Continuous,
}

/// Error related to market data object
#[derive(Debug)]
pub struct DiscountError;

impl std::fmt::Display for DiscountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "discount error: the cash flow currency does not match the discounter currency"
        )
    }
}

impl std::error::Error for DiscountError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// The `Discounter` trait provides a method for calculating discount factors.
/// This could be applied to falt raters, rate curves, or more complex models.
pub trait Discounter {
    /// Calculate the factor to discount a cash flow at `pay_date` to `today`.
    fn discount_factor(&self, today: Date, pay_date: Date) -> f64;

    /// Each discounter must belong to a currency, i.e. only cash flows in
    /// the same currency can be discounted.
    fn currency(&self) -> Currency;

    /// Discount given cash flow
    fn discount_cash_flow(&self, cf: &CashFlow, today: Date) -> Result<CashAmount, DiscountError> {
        if self.currency() == cf.amount.currency {
            let amount = self.discount_factor(today, cf.date) * cf.amount.amount;
            Ok(CashAmount {
                amount,
                currency: cf.amount.currency,
            })
        } else {
            Err(DiscountError)
        }
    }

    /// Discount given cash flow stream
    fn discount_cash_flow_stream(
        &self,
        cf_stream: &[CashFlow],
        today: Date,
    ) -> Result<CashAmount, DiscountError> {
        let mut amount = CashAmount {
            amount: 0.0,
            currency: self.currency(),
        };
        for cf in cf_stream {
            if self.currency() == cf.amount.currency {
                amount.amount += self.discount_factor(today, cf.date) * cf.amount.amount;
            } else {
                return Err(DiscountError);
            }
        }
        Ok(amount)
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub struct FlatRate {
    pub rate: f64,
    day_count_conv: DayCountConv,
    compounding: Compounding,
    currency: Currency,
}

impl FlatRate {
    /// Constructor of `FlatRate`
    pub fn new(
        rate: f64,
        day_count_conv: DayCountConv,
        compounding: Compounding,
        currency: Currency,
    ) -> FlatRate {
        FlatRate {
            rate,
            day_count_conv,
            compounding,
            currency,
        }
    }
}

impl Discounter for FlatRate {
    fn discount_factor(&self, today: Date, pay_date: Date) -> f64 {
        let yf = self
            .day_count_conv
            .year_fraction(today, pay_date, None, None)
            .unwrap();
        match self.compounding {
            Compounding::Simple => 1. / (1. + self.rate * yf),
            Compounding::Annual => (1. + self.rate).powf(-yf),
            Compounding::SemiAnnual => (1. + 0.5 * self.rate).powf(-2. * yf),
            Compounding::Quarterly => (1. + 0.25 * self.rate).powf(-4. * yf),
            Compounding::Monthly => (1. + self.rate / 12.).powf(-12. * yf),
            Compounding::Continuous => (-self.rate * yf).exp(),
        }
    }

    fn currency(&self) -> Currency {
        self.currency
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time_period::TimePeriod;
    use std::f64;
    use std::str::FromStr;

    #[test]
    fn compounding_methods() {
        let tol = 1e-11;
        let curr = Currency::from_str("EUR").unwrap();
        let rate = FlatRate {
            rate: 0.05,
            day_count_conv: DayCountConv::Act365,
            compounding: Compounding::Annual,
            currency: curr,
        };
        let start_date = Date::from_calendar_date(2019, time::Month::December, 16).unwrap();
        let end_date = start_date + TimePeriod::from_str("6M").unwrap();
        let yf: f64 = DayCountConv::Act365
            .year_fraction(start_date, end_date, None, None)
            .unwrap();
        assert_fuzzy_eq!(
            rate.discount_factor(start_date, end_date),
            f64::powf(1.0 + 0.05, -yf),
            tol
        );

        let rate = FlatRate {
            rate: 0.05,
            day_count_conv: DayCountConv::Act365,
            compounding: Compounding::SemiAnnual,
            currency: curr,
        };
        assert_fuzzy_eq!(
            rate.discount_factor(start_date, end_date),
            f64::powf(1.0 + 0.025, -yf * 2.),
            tol
        );

        let rate = FlatRate {
            rate: 0.05,
            day_count_conv: DayCountConv::Act365,
            compounding: Compounding::Quarterly,
            currency: curr,
        };
        assert_fuzzy_eq!(
            rate.discount_factor(start_date, end_date),
            f64::powf(1.0 + 0.0125, -yf * 4.),
            tol
        );

        let rate = FlatRate {
            rate: 0.05,
            day_count_conv: DayCountConv::Act365,
            compounding: Compounding::Monthly,
            currency: curr,
        };
        println!(
            "{},{}",
            rate.discount_factor(start_date, end_date),
            f64::powf(1.0 + 0.05 / 12., -yf * 12.)
        );
        assert_fuzzy_eq!(
            rate.discount_factor(start_date, end_date),
            f64::powf(1.0 + 0.05 / 12., -yf * 12.),
            tol
        );

        let rate = FlatRate {
            rate: 0.05,
            day_count_conv: DayCountConv::Act365,
            compounding: Compounding::Continuous,
            currency: curr,
        };
        assert_fuzzy_eq!(
            rate.discount_factor(start_date, end_date),
            f64::exp(-0.05 * yf),
            tol
        );

        let rate = FlatRate {
            rate: 0.05,
            day_count_conv: DayCountConv::Act365,
            compounding: Compounding::Simple,
            currency: curr,
        };
        assert_fuzzy_eq!(
            rate.discount_factor(start_date, end_date),
            1. / (1. + 0.05 * yf),
            tol
        );
    }

    #[test]
    fn discounting() {
        let tol = 1e-11;
        let curr = Currency::from_str("EUR").unwrap();
        let rate = FlatRate {
            rate: 0.05,
            day_count_conv: DayCountConv::Act365,
            compounding: Compounding::Continuous,
            currency: curr,
        };
        let cash_flows = vec![
            CashFlow::new(
                100.,
                curr,
                Date::from_calendar_date(2021, time::Month::April, 1).unwrap(),
            ),
            CashFlow::new(
                100.,
                curr,
                Date::from_calendar_date(2021, time::Month::October, 1).unwrap(),
            ),
            CashFlow::new(
                100.,
                curr,
                Date::from_calendar_date(2022, time::Month::April, 1).unwrap(),
            ),
            CashFlow::new(
                100.,
                curr,
                Date::from_calendar_date(2022, time::Month::October, 3).unwrap(),
            ),
        ];
        let today = Date::from_calendar_date(2019, time::Month::October, 1).unwrap();
        assert_fuzzy_eq!(
            rate.discount_cash_flow(&cash_flows[0], today)
                .unwrap()
                .amount,
            100. * f64::exp(-0.05 * (366. + 182.) / 365.),
            tol
        );
        assert_fuzzy_eq!(
            rate.discount_cash_flow(&cash_flows[1], today)
                .unwrap()
                .amount,
            100. * f64::exp(-0.05 * (366. + 365.) / 365.),
            tol
        );
        assert_fuzzy_eq!(
            rate.discount_cash_flow(&cash_flows[2], today)
                .unwrap()
                .amount,
            100. * f64::exp(-0.05 * (366. + 365. + 182.) / 365.),
            tol
        );
        assert_fuzzy_eq!(
            rate.discount_cash_flow(&cash_flows[3], today)
                .unwrap()
                .amount,
            100. * f64::exp(-0.05 * (366. + 2. * 365. + 2.) / 365.),
            tol
        );
        let sum = 100.
            * (f64::exp(-0.05 * (366. + 182.) / 365.)
                + f64::exp(-0.05 * (366. + 365.) / 365.)
                + f64::exp(-0.05 * (366. + 365. + 182.) / 365.)
                + f64::exp(-0.05 * (366. + 2. * 365. + 2.) / 365.));
        assert_fuzzy_eq!(
            rate.discount_cash_flow_stream(&cash_flows, today)
                .unwrap()
                .amount,
            sum,
            tol
        );
    }
}
