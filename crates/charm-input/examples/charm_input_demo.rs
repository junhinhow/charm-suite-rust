//! Demo for charm-input.

use charm_input::Date;
use chrono::NaiveDate;

fn main() {
    let mut date = Date::new(NaiveDate::from_ymd_opt(2026, 10, 22).expect("valid"));
    date.set_size(40, 8);
    date.focus();
    println!("{}", date.view());
}
