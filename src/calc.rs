use std::str::FromStr;
use rust_decimal::prelude::{Decimal, ToPrimitive};

pub fn count_new_basic(target_price: i32, current_discounted: i32, current_basic: i32) -> (i32, i32) {
    let part = Decimal::from(current_discounted) / Decimal::from(current_basic);
    let part = correct_part(part);

    let new_discounted = Decimal::from(target_price) / _d("0.97");
    let new_base = (new_discounted / part).round().to_i32().expect("smth wrong");

    let mut new_price = correct(new_base, part, target_price, 0);
    let current_basic_rub = Decimal::from(current_basic) / _d("100");

    if (Decimal::from(new_price) / current_basic_rub) <= _d("0.3") {
        new_price = (current_basic_rub * _d("0.4")).round().to_i32().expect("smth wrong");
    };

    ((Decimal::from(new_price) * part).floor().to_i32().expect("smth wrong"), new_price)
}

fn correct_part(start: Decimal) -> Decimal {
    let rounded_part_1000 = (start * _d("1000")).round();
    let last_digit = rounded_part_1000.to_i64().expect("smth wrong") % 10;

    if last_digit == 5 {
        rounded_part_1000 / _d("1000")
    } else {
        start
    }
}

fn correct(base: i32, part: Decimal, target: i32, base_diff: i32) -> i32 {
    let base = base + base_diff;
    let discounted = (Decimal::from(base) * part).floor();
    let m_target = (discounted * _d("0.97")).floor().to_i32().expect("smth wrong");

    if m_target > target && base_diff <= 0 {
        correct(base, part, target, -1)
    } else if m_target < target && base_diff >= 0 {
        correct(base, part, target, 1)
    } else {
        base
    }
}

fn _d(s: &str) -> Decimal {
    return Decimal::from_str(s).expect("smth wrong")
}
