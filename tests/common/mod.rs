use csv::ReaderBuilder;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Record {
    #[serde(rename = "Open Time")]
    pub open_time: i64,
    #[serde(rename = "Open Price")]
    pub open_price: Decimal,
    #[serde(rename = "High Price")]
    pub high_price: Decimal,
    #[serde(rename = "Low Price")]
    pub low_price: Decimal,
    #[serde(rename = "Close Price")]
    pub close_price: Decimal,
    #[serde(rename = "Volume")]
    pub volume: Decimal,
    #[serde(rename = "SMA (10)", with = "rust_decimal::serde::str_option")]
    pub sma: Option<Decimal>,
    #[serde(rename = "VWAP (10)", with = "rust_decimal::serde::str_option")]
    pub vwap: Option<Decimal>,
    #[serde(rename = "EMA (10)", with = "rust_decimal::serde::str_option")]
    pub ema: Option<Decimal>,
    #[serde(rename = "KAMA (5, 2, 4)", with = "rust_decimal::serde::str_option")]
    pub kama: Option<Decimal>,
    #[serde(rename = "WMA (5)", with = "rust_decimal::serde::str_option")]
    pub wma: Option<Decimal>,
}

pub fn read_csv() -> Vec<Record> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path("tests/data/ETHUSDT-1D.csv")
        .unwrap();

    reader.deserialize().map(|x| x.unwrap()).collect()
}
