use ferrous_ta::*;
use rust_decimal::RoundingStrategy;

mod common;
use common::read_csv;

#[test]
fn test_wma_5() {
    const PERIODS: usize = 5;

    // let mut wma = WMA::<_, PERIODS>::new();

    for i in read_csv() {
        // TODO!!!
        assert_eq!(true, true);
    }
}
