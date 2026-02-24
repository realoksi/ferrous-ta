use ferrous_ta::*;

use csv::ReaderBuilder;
use rust_decimal::{Decimal, RoundingStrategy};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "t")]
    open_time: i64,
    #[serde(rename = "o")]
    open_price: Decimal,
    #[serde(rename = "h")]
    high_price: Decimal,
    #[serde(rename = "l")]
    low_price: Decimal,
    #[serde(rename = "c")]
    close_price: Decimal,
    #[serde(rename = "v")]
    volume: Decimal,
    #[serde(with = "rust_decimal::serde::str_option")]
    sma: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::str_option")]
    vwap: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::str_option")]
    kama: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::str_option")]
    wma: Option<Decimal>,
}

#[test]
fn test_kama_10() {
    todo!();
}
