pub fn count_new_basic(target_price: i32, current_discounted: i32, current_basic: i32) -> (i32, i32) {
    let part = current_discounted as f64 / current_basic as f64;
    let part = correct_part(part);

    let new_discounted = target_price as f64 / 0.97;
    let new_base = (new_discounted / part) as i32;

    let mut new_price = correct(new_base, part, target_price);
    let current_basic_rub = current_basic / 100;
    if (new_price as f64 / current_basic_rub as f64) < 0.3 {
        new_price = (current_basic_rub as f64 * 0.4).round() as i32
    };

    ((new_price as f64 * part).floor() as i32, new_price)
}

fn correct_part(start: f64) -> f64 {
    let rounded_part_1000 = (start * 1000.0).round();
    let last_digit = rounded_part_1000 as i64 % 10;

    if last_digit == 5 {
        rounded_part_1000 / 1000.0
    } else {
        start
    }
}

fn correct(base: i32, part: f64, target: i32) -> i32 {
    let discounted = (base as f64 * part).floor();
    let m_target = (discounted * 0.97).floor() as i32;

    if m_target == target {
        base
    } else if m_target > target {
        correct(base - 1, part, target)
    } else {
        correct(base + 1, part, target)
    }
}
