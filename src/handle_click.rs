use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use evdev::InputEvent;

use crate::{
    cli::{Cli, Mode},
    conversions::to_nanos,
    shared_state::SharedState,
};

pub fn handle_click(
    event: &InputEvent,
    state: &SharedState,
    cli: &Cli,
    spammer_key: &str,
    enabled: &Arc<AtomicBool>,
    click_delay: f64,
    start_delay: f64,
    last_click: &mut Instant,
) {
    if event.value() == 1 {
        state.click_counter.fetch_add(1, Ordering::Relaxed);
        if enabled.load(Ordering::Relaxed) {
            match cli.mode {
                Mode::Hold => {
                    state
                        .get_spammer(spammer_key)
                        .unwrap()
                        .lock()
                        .unwrap()
                        .click_delay_ns
                        .store(to_nanos(click_delay), Ordering::Relaxed);
                    state.enable_spammer(spammer_key).unwrap();
                }
                Mode::Toggle => {
                    if cli.disable_on_click {
                        enabled.store(false, Ordering::Relaxed);
                        state.disable_spammer(spammer_key).unwrap();
                    }
                }
                Mode::Both => {
                    state
                        .get_spammer(spammer_key)
                        .unwrap()
                        .lock()
                        .unwrap()
                        .click_delay_ns
                        .store(to_nanos(click_delay), Ordering::Relaxed);
                    *last_click = Instant::now();
                    state.toggle_spammer(spammer_key).unwrap();
                }
                Mode::Always => unreachable!(),
            }
        }
    } else if event.value() == 0 && enabled.load(Ordering::Relaxed) {
        match cli.mode {
            Mode::Hold => {
                state.disable_spammer(spammer_key).unwrap();
            }

            Mode::Toggle => (),
            Mode::Both => {
                if last_click.elapsed() > Duration::from_secs_f64(start_delay) {
                    state.disable_spammer(spammer_key).unwrap();
                }
            }
            Mode::Always => unreachable!(),
        }
    }
}
