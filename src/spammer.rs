use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::{self, sleep},
    time::Duration,
};

pub struct Spammer {
    enabled: Arc<AtomicBool>,
    device: Arc<uinput_rs::Device>,
    key: (u64, u64),
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
    ) -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(false)),
            key,
            device,
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

            thread::spawn(move || {
                sleep(start_delay);

                while enabled.load(Ordering::Relaxed) {
                    device.emit_key_code_silent(key, 1);
                    device.sync_silent();
                    sleep(Duration::from_nanos(click_delay_ns.load(Ordering::Relaxed)));
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
