use std::{
    fmt::Display,
    io::{Write, stdout},
    sync::atomic::Ordering,
    thread::{self, sleep},
    time::Duration,
};

use ringbuffer::{AllocRingBuffer, RingBuffer};

use crate::shared_state::SharedState;

pub fn print_beginning<T: Display, U: Display>(
    device_name: &T,
    start_delay_left: &U,
    start_delay_right: &U,
    left_click_delay: &f64,
    right_click_delay: &f64,
    fast_click_delay: &f64,
) {
    println!("Selected device name: {device_name}");
    println!("Start delay: Left {start_delay_left} | Right {start_delay_right}");
    println!(
        "Cps target: Left {} | Right {} | Fast {}",
        1.0 / left_click_delay,
        1.0 / right_click_delay,
        1.0 / fast_click_delay,
    );
    println!(
        "Click delay (seconds): Left {left_click_delay} | Right {right_click_delay} | Fast {fast_click_delay}"
    );
}

pub fn print_state<T: Display, U: Display, V: Display, W: Display>(
    left_enabled: T,
    right_enabled: U,
    fast_enabled: V,
    cps: W,
) {
    let longest = format!(
        "left: false | right: false | fast: false | cps: {}",
        u32::MAX
    )
    .len();
    print!("\r{}", " ".repeat(longest));
    print!("\rLeft: {left_enabled} | Right: {right_enabled} | Fast: {fast_enabled} | cps: {cps}");
    let _ = stdout().flush();
}

pub fn spawn_status_thread(state: SharedState, update_delay: u64) {
    let cps = state.cps.clone();
    thread::spawn(move || {
        let delay = Duration::from_millis(update_delay);
        let mut buckets: AllocRingBuffer<u64> =
            AllocRingBuffer::new((1000 / update_delay) as usize);
        loop {
            sleep(delay);
            let clicks = state.click_counter.load(Ordering::Relaxed);
            state.click_counter.fetch_sub(clicks, Ordering::Relaxed);
            buckets.enqueue(clicks);
            cps.store(buckets.iter().sum::<u64>(), Ordering::Relaxed);
        }
    });

    thread::spawn(move || {
        let delay = Duration::from_millis(10);

        loop {
            sleep(delay);
            print_state(
                state.left_enabled.load(Ordering::Relaxed),
                state.right_enabled.load(Ordering::Relaxed),
                state.fast_enabled.load(Ordering::Relaxed),
                state.cps.load(Ordering::Relaxed),
            );
        }
    });
}
