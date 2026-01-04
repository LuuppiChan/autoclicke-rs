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
use uinput_rs::key_codes::BTN_LEFT;

use crate::conversions::to_nanos;

pub struct Spammer {
    pub click_delay_ns: Arc<AtomicU64>,
    pub key: (u64, u64),
    pub randomize: bool,
    pub deviation: f64,
    pub start_delay: Duration,
    pub click_counter: Option<Arc<AtomicU64>>,
    pub quantity: usize,
    enabled: Arc<AtomicBool>,
    device: Arc<uinput_rs::Device>,
}

impl Spammer {
    /// Create new spammer.
    pub fn new(device: Arc<uinput_rs::Device>) -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(false)),
            click_delay_ns: Arc::new(to_nanos(0.05).into()),
            start_delay: Duration::from_millis(100),
            key: BTN_LEFT,
            randomize: false,
            deviation: 0.3,
            click_counter: None,
            quantity: 1,
            device,
        }
    }

    pub fn click_counter(mut self, counter: Arc<AtomicU64>) -> Self {
        self.click_counter = Some(counter);
        self
    }

    pub fn key(mut self, key: (u64, u64)) -> Self {
        self.key = key;
        self
    }

    pub fn click_delay(mut self, delay: Duration) -> Self {
        self.set_click_delay_dur(delay);
        self
    }

    pub fn start_delay(mut self, start_delay: Duration) -> Self {
        self.start_delay = start_delay;
        self
    }

    pub fn randomize(mut self, randomize: bool) -> Self {
        self.randomize = randomize;
        self
    }

    /// Clamps if a given value is not in the from 0 to 1
    pub fn deviation(mut self, deviation: f64) -> Self {
        self.deviation = deviation.clamp(0.0, 1.0);
        self
    }

    pub fn quantity<T: Into<usize>>(mut self, quantity: T) -> Self {
        self.quantity = quantity.into();
        self
    }

    pub fn set_click_delay_dur(&mut self, delay: Duration) {
        self.click_delay_ns
            .store(delay.as_nanos() as u64, Ordering::Relaxed);
    }

    pub fn set_click_delay_f(&mut self, delay: f64) {
        self.click_delay_ns
            .store(to_nanos(delay), Ordering::Relaxed);
    }

    pub fn set_click_delay_ns(&mut self, delay: u64) {
        self.click_delay_ns.store(delay, Ordering::Relaxed);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn enable(&self) {
        if self.is_enabled() {
            return;
        }

        self.enabled.store(true, Ordering::Relaxed);
        for _ in 0..self.quantity {
            let start_delay = self.start_delay;
            let enabled = self.enabled.clone();
            let device = self.device.clone();
            let key = self.key;
            let click_counter = self.click_counter.clone();
            // hacky, but it will work for now
            // Maybe turn the click delays into a vector of enum members
            // then match
            // Or just give the specified index in that vector which has the wanted delay
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
                    if let Some(click_counter) = &click_counter {
                        click_counter.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });
        }
    }

    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }
}
