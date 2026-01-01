use core::f64;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::{self, sleep},
    time::Duration,
};

use rand::Rng;

pub struct Spammer {
    enabled: Arc<AtomicBool>,
    device: Arc<uinput_rs::Device>,
    key: (u64, u64),
    randomize: bool,
    deviation: f64,
    start_delay: Duration,
    click_delay_ns: Arc<AtomicU64>,
    click_counter: Arc<AtomicU64>,
}

impl Spammer {
    /// Create new spammer.
    pub fn new(
        device: Arc<uinput_rs::Device>,
        key: (u64, u64),
        start_delay: Duration,
        click_delay_ns: Arc<AtomicU64>,
        click_counter: Arc<AtomicU64>,
        randomize: bool,
        deviation: f64,
    ) -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(false)),
            device,
            key,
            randomize,
            deviation,
            start_delay,
            click_delay_ns,
            click_counter,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn enable<T: Into<usize>>(&self, quantity: T) {
        if self.is_enabled() {
            return;
        }

        self.enabled.store(true, Ordering::Relaxed);
        for _ in 0..quantity.into() {
            let start_delay = self.start_delay;
            let enabled = self.enabled.clone();
            let device = self.device.clone();
            let key = self.key;
            let click_counter = self.click_counter.clone();
            let click_delay_ns = self.click_delay_ns.clone();
            let randomize = self.randomize;
            let deviation = self.deviation;

            thread::spawn(move || {
                sleep(start_delay);
                let mut rng = rand::rng();

                while enabled.load(Ordering::Relaxed) {
                    device.emit_key_code_silent(key, 1);
                    device.sync_silent();

                    let nanos = click_delay_ns.load(Ordering::Relaxed);
                    let delay = if randomize {
                        let base = nanos as f64 / 1_000_000_000.0;
                        let result =
                            rng.random_range(base * (1.0 - deviation)..=base * (1.0 + deviation));
                        Duration::from_secs_f64(result)
                    } else {
                        Duration::from_nanos(nanos)
                    };
                    sleep(delay);

                    device.emit_key_code_silent(key, 0);
                    device.sync_silent();
                    click_counter.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    }

    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }
}
