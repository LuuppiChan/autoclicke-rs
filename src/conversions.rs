use std::time::Duration;

pub fn to_nanos<T: Into<f64>>(number: T) -> u64 {
    Duration::from_secs_f64(number.into()).as_nanos() as u64
}
