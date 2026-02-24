use ferrous_ta::*;
use rust_decimal::RoundingStrategy;

mod common;
use common::read_csv;

#[test]
fn test_sma_10() {
    const PERIODS: usize = 10;

    let mut sma = SMA::<_, PERIODS>::new();

    for i in read_csv() {
        if let Some(value) = sma.step(i.close_price) {
            assert_eq!(
                i.sma.unwrap(),
                value.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
            );
        }
    }
}
