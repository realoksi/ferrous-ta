use ferrous_ta::{Volatility, ATR};

use csv::ReaderBuilder;
use rust_decimal::{dec, Decimal};

/*
Calculated manually according to J. Welles Wilder Jr.'s original ATR formulation, as defined in "New
Concepts in Technical Trading Systems (1978)" using a 14-period look-back and Wilder's smoothing
method.

    High     Low      Close    TR      ATR
1:  2999.11, 2721.04, 2802.75,    n/a,    n/a
2:  3032.08, 2783.17, 2993.49, 248.91,    n/a
3:  3215.56, 2983.71, 3189.53, 231.85,    n/a
4:  3237.85, 3067.99, 3129.94, 169.86,    n/a
5:  3191.86, 2987.03, 3019.60, 204.83,    n/a
6:  3063.78, 3008.96, 3036.78,  54.82,    n/a
7:  3148.73, 2924.62, 3062.31, 224.11,    n/a
8:  3178.36, 3037.15, 3124.14, 141.21,    n/a
9:  3397.52, 3091.46, 3319.30, 306.06,    n/a
10: 3445.49, 3290.14, 3321.51, 155.35,    n/a
11: 3328.78, 3146.02, 3236.25, 182.76,    n/a
12: 3264.30, 3047.85, 3083.62, 216.45,    n/a
13: 3137.49, 3075.99, 3112.60,  61.50,    n/a
14: 3128.00, 3030.00, 3065.01,  98.00,    n/a
15: 3176.40, 2864.36, 2965.68, 312.04, 186.27
16: 2979.11, 2882.44, 2963.89,  96.67, 179.87
*/

#[test]
fn test_atr_wilder_14() {
    const PERIODS: usize = 14;
    const ANSWERS: [Decimal; 2] = [dec!(186.27), dec!(179.87)];

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path("tests/data/ETHUSDT-1D.csv")
        .unwrap();

    let mut atr = ATR::new(PERIODS);

    let mut index = 0;
    for record in reader.records() {
        let record = record.unwrap();

        let high_price: Decimal = record[2].parse().unwrap();
        let low_price = record[3].parse().unwrap();
        let close_price = record[4].parse().unwrap();

        if let Some(a) = atr.push([high_price, low_price, close_price]) {
            assert_eq!(a.round_dp(2), ANSWERS[index - PERIODS]);
        }

        index += 1;

        if index > PERIODS {
            break;
        }
    }
}
