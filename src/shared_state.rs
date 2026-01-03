use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64},
    },
    time::Duration,
};

use crate::{cli::Cli, conversions::to_nanos};

pub fn init(cli: &Cli) -> SharedState {
    SharedState {
        left_enabled: Arc::new(AtomicBool::new(cli.enable_left)),
        right_enabled: Arc::new(AtomicBool::new(cli.enable_right)),
        fast_enabled: Arc::new(AtomicBool::new(cli.enable_fast)),
        click_counter: Arc::new(AtomicU64::new(0)),
        left_click_delay_ns: Arc::new(AtomicU64::new(if cli.enable_fast {
            to_nanos(cli.fast_click_delay)
        } else {
            to_nanos(cli.left_click_delay)
        })),
        right_click_delay_ns: Arc::new(AtomicU64::new(
            Duration::from_secs_f64(cli.right_click_delay).as_nanos() as u64,
        )),
        cps: Arc::new(AtomicU64::new(0)),
    }
}

#[derive(Clone)]
pub struct SharedState {
    pub left_enabled: Arc<AtomicBool>,
    pub right_enabled: Arc<AtomicBool>,
    pub fast_enabled: Arc<AtomicBool>,
    pub click_counter: Arc<AtomicU64>,
    pub cps: Arc<AtomicU64>,
    pub left_click_delay_ns: Arc<AtomicU64>,
    pub right_click_delay_ns: Arc<AtomicU64>,
}
