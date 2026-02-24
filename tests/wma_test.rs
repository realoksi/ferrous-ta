use ferrous_ta::*;

use csv::ReaderBuilder;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "t")]
    open_time: i64,
    #[serde(rename = "o")]
    open_price: f64,
    #[serde(rename = "h")]
    high_price: f64,
    #[serde(rename = "l")]
    low_price: f64,
    #[serde(rename = "c")]
    close_price: f64,
    #[serde(rename = "v")]
    volume: f64,
    sma: Option<f64>,
    vwap: Option<f64>,
    kama: Option<f64>,
    wma: Option<f64>,
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
            assert_eq!(record.wma.unwrap(), (value * 100.0).round() / 100.0);
        }
    }
}
