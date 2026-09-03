use catsquad_log::prelude::*;
use catsquad_shared::Uuid;
use std::{ascii::AsciiExt, char};

pub fn get_time_ns() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    time.as_nanos()
}

pub fn get_time_micro() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    time.as_micros()
}

pub fn rng_str(len: usize) -> String {
    use rand::distr::SampleString;
    rand::distr::Alphanumeric.sample_string(&mut rand::rng(), len)
}
