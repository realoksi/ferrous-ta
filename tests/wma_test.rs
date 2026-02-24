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
fn test_wma_5() {
    const PERIODS: usize = 5;

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path("tests/data/ETHUSDT-1D.csv")
        .unwrap();

    let mut wma = WMA::<_, PERIODS>::new();

    for record in reader.deserialize() {
        let record: Record = record.unwrap();

        if let Some(value) = wma.step(record.close_price) {
            assert_eq!(
                record.wma.unwrap(),
                value.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
            );
        }
    }
}
