use chrono::{Datelike};
use crate::app_helper_structs::{Season};
pub struct Utils {
}
impl Utils {
    pub fn get_year() -> i64 {
        chrono::Utc::now().year() as i64
    }
    pub fn get_season() -> Season {
        let current_date = chrono::Utc::now();
        let month = current_date.month();
        match month {
            1 | 2 | 3 => Season::WINTER,
            4 | 5 | 6 => Season::SPRING,
            7 | 8 | 9 => Season::SUMMER,
            10 | 11 | 12 => Season::FALL,
            _ => unimplemented!(),
        }
    }
}