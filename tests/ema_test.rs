use ferrous_ta::*;
use rust_decimal::RoundingStrategy;

mod common;
use common::read_csv;

#[test]
fn test_ema_10() {
    const PERIODS: i32 = 10;

    let mut ema = EMA::new(None, PERIODS, 2);

    for i in read_csv() {
        let value = ema.step(i.close_price);

        assert_eq!(
            i.ema.unwrap(),
            value.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
        );
    }
}
