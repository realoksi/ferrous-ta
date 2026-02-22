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
}

#[test]
fn test_sma_10() {
    const PERIODS: usize = 10;

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path("tests/data/ETHUSDT-1D.csv")
        .unwrap();

    let mut sma = SMA::<_, PERIODS>::new();

    for record in reader.deserialize() {
        let record: Record = record.unwrap();

        if let Some(value) = sma.step(record.close_price) {
            assert_eq!(
                record.sma.unwrap(),
                value.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
            );
        }
    }
}
