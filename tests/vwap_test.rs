use ferrous_ta::*;
use rust_decimal::RoundingStrategy;

mod common;
use common::read_csv;

#[test]
fn test_vwap_10() {
    const PERIODS: usize = 10;

    let mut vwap = VWAP::<_, PERIODS>::new();

    for i in read_csv() {
        if let Some(value) = vwap.step([i.close_price, i.volume]) {
            assert_eq!(
                i.vwap.unwrap(),
                value.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
            );
        }
    }
}
