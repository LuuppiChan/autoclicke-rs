use std::{
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

use uinput_rs::key_codes::{BTN_LEFT, BTN_MIDDLE, BTN_RIGHT};

use crate::{
    cli::Mode,
    conversions::to_nanos,
    interface::{print_beginning, spawn_status_thread},
    spammer::Spammer,
};

mod always;
mod cli;
mod conversions;
mod interface;
mod shared_state;
mod spammer;
mod virtual_device;

fn main() {
    let cli = cli::parse();
    let virtual_device = Arc::new(virtual_device::get());

    // let left_enabled = Arc::new(AtomicBool::new(cli.enable_left));
    // let right_enabled = Arc::new(AtomicBool::new(cli.enable_right));
    // let fast_enabled = Arc::new(AtomicBool::new(cli.enable_fast));
    // let click_counter = Arc::new(AtomicU64::new(0));
    // let left_click_delay_ns = Arc::new(AtomicU64::new(
    //     Duration::from_secs_f64(cli.left_click_delay).as_nanos() as u64,
    // ));
    // let right_click_delay_ns = Arc::new(AtomicU64::new(
    //     Duration::from_secs_f64(cli.right_click_delay).as_nanos() as u64,
    // ));
    let state = shared_state::init(&cli);
    let mut last_left_click = Instant::now();
    let mut last_right_click = Instant::now();

    let left_spammer = Spammer::new(
        virtual_device.clone(),
        BTN_LEFT,
        Duration::from_secs_f64(cli.start_delay_left),
        state.clone(),
        state.left_click_delay_ns.clone(),
        cli.randomize,
        cli.deviation,
    );
    let right_spammer = Spammer::new(
        virtual_device.clone(),
        BTN_RIGHT,
        Duration::from_secs_f64(cli.start_delay_right),
        state.clone(),
        state.right_click_delay_ns.clone(),
        cli.randomize,
        cli.deviation,
    );

    if cli.mode == Mode::Always {
        always::run(cli, left_spammer, right_spammer);
    } else if cli.mode == Mode::Toggle {
        if cli.enable_fast {
            state
                .left_click_delay_ns
                .store(to_nanos(cli.fast_click_delay), Ordering::Relaxed);
        }
        if cli.enable_left {
            left_spammer.enable(cli.spammers);
        }
        if cli.enable_right {
            right_spammer.enable(cli.spammers);
        }
    }

    let mut device = evdev::Device::open(
        cli.mouse_path
            .expect("Please give a mouse path. It's required if you use Hold or Toggle modes."),
    )
    .expect("Error opening device");

    print_beginning(
        &device.name().unwrap_or_default(),
        &cli.start_delay_left,
        &cli.start_delay_right,
        &cli.left_click_delay,
        &cli.right_click_delay,
        &cli.fast_click_delay,
    );
    spawn_status_thread(state.clone(), cli.update_delay);

    device.grab().expect("Error grabbing selected device.");

    while let Ok(events) = device.fetch_events() {
        for event in events {
            match event.event_type().0 {
                1 => match event.code() {
                    // left
                    272 => {
                        virtual_device.emit_key_code_silent(BTN_LEFT, event.value());

                        if event.value() == 1 {
                            state.click_counter.fetch_add(1, Ordering::Relaxed);
                            if state.left_enabled.load(Ordering::Relaxed) {
                                match cli.mode {
                                    Mode::Hold => {
                                        state.left_click_delay_ns.store(
                                            to_nanos(
                                                if state.fast_enabled.load(Ordering::Relaxed) {
                                                    cli.fast_click_delay
                                                } else {
                                                    cli.left_click_delay
                                                },
                                            ),
                                            Ordering::Relaxed,
                                        );
                                        left_spammer.enable(cli.spammers);
                                    }
                                    Mode::Toggle => {
                                        if cli.disable_on_click {
                                            state.left_enabled.store(false, Ordering::Relaxed);
                                            left_spammer.disable();
                                        }
                                    }
                                    Mode::Both => {
                                        if left_spammer.is_enabled() {
                                            left_spammer.disable();
                                        } else {
                                            state.left_click_delay_ns.store(
                                                to_nanos(
                                                    if state.fast_enabled.load(Ordering::Relaxed) {
                                                        cli.fast_click_delay
                                                    } else {
                                                        cli.left_click_delay
                                                    },
                                                ),
                                                Ordering::Relaxed,
                                            );
                                            last_left_click = Instant::now();
                                            left_spammer.enable(cli.spammers);
                                        }
                                    }
                                    Mode::Always => unreachable!(),
                                }
                            }
                        } else if event.value() == 0 && state.left_enabled.load(Ordering::Relaxed) {
                            match cli.mode {
                                Mode::Hold => left_spammer.disable(),
                                Mode::Toggle => (),
                                Mode::Both => {
                                    if last_left_click.elapsed()
                                        > Duration::from_secs_f64(cli.start_delay_left)
                                    {
                                        left_spammer.disable();
                                    }
                                }
                                Mode::Always => unreachable!(),
                            }
                        }
                    }
                    // right
                    273 => {
                        virtual_device.emit_key_code_silent(BTN_RIGHT, event.value());

                        if event.value() == 1 {
                            state.click_counter.fetch_add(1, Ordering::Relaxed);
                            if state.right_enabled.load(Ordering::Relaxed) {
                                match cli.mode {
                                    Mode::Hold => {
                                        state.right_click_delay_ns.store(
                                            to_nanos(cli.right_click_delay),
                                            Ordering::Relaxed,
                                        );
                                        right_spammer.enable(cli.spammers);
                                    }
                                    Mode::Toggle => {
                                        if cli.disable_on_click {
                                            state.right_enabled.store(false, Ordering::Relaxed);
                                            right_spammer.disable();
                                        }
                                    }
                                    Mode::Both => {
                                        if right_spammer.is_enabled() {
                                            right_spammer.disable();
                                        } else {
                                            state.right_click_delay_ns.store(
                                                to_nanos(cli.right_click_delay),
                                                Ordering::Relaxed,
                                            );
                                            last_right_click = Instant::now();
                                            right_spammer.enable(cli.spammers);
                                        }
                                    }
                                    Mode::Always => unreachable!(),
                                }
                            }
                        } else if event.value() == 0 && state.right_enabled.load(Ordering::Relaxed)
                        {
                            match cli.mode {
                                Mode::Hold => right_spammer.disable(),
                                Mode::Toggle => (),
                                Mode::Both => {
                                    if last_right_click.elapsed()
                                        > Duration::from_secs_f64(cli.start_delay_right)
                                    {
                                        right_spammer.disable();
                                    }
                                }
                                Mode::Always => unreachable!(),
                            }
                        }
                    }
                    // middle
                    274 => {
                        virtual_device.emit_key_code_silent(BTN_MIDDLE, event.value());
                        if event.value() == 1 && state.left_enabled.load(Ordering::Relaxed) {
                            state.fast_enabled.fetch_not(Ordering::Relaxed);
                            state.left_click_delay_ns.store(
                                Duration::from_secs_f64(
                                    if state.fast_enabled.load(Ordering::Relaxed) {
                                        cli.fast_click_delay
                                    } else {
                                        cli.left_click_delay
                                    },
                                )
                                .as_nanos() as u64,
                                Ordering::Relaxed,
                            );
                        }
                    }
                    // side
                    275 => {
                        if event.value() == 1 {
                            state.left_enabled.fetch_not(Ordering::Relaxed);
                            if state.left_enabled.load(Ordering::Relaxed) {
                                match cli.mode {
                                    Mode::Hold => (),
                                    Mode::Toggle => {
                                        state.left_click_delay_ns.store(
                                            to_nanos(
                                                if state.fast_enabled.load(Ordering::Relaxed) {
                                                    cli.fast_click_delay
                                                } else {
                                                    cli.left_click_delay
                                                },
                                            ),
                                            Ordering::Relaxed,
                                        );
                                        left_spammer.enable(cli.spammers);
                                    }
                                    Mode::Both => (),
                                    Mode::Always => unreachable!(),
                                }
                            } else {
                                match cli.mode {
                                    Mode::Hold => left_spammer.disable(),
                                    Mode::Toggle => left_spammer.disable(),
                                    Mode::Both => left_spammer.disable(),
                                    Mode::Always => unreachable!(),
                                }
                            }
                        }
                    }
                    // extra
                    276 => {
                        if event.value() == 1 {
                            state.right_enabled.fetch_not(Ordering::Relaxed);
                            if state.right_enabled.load(Ordering::Relaxed) {
                                match cli.mode {
                                    Mode::Hold => (),
                                    Mode::Toggle => {
                                        state.right_click_delay_ns.store(
                                            to_nanos(cli.right_click_delay),
                                            Ordering::Relaxed,
                                        );
                                        right_spammer.enable(cli.spammers);
                                    }
                                    Mode::Both => (),
                                    Mode::Always => unreachable!(),
                                }
                            } else {
                                match cli.mode {
                                    Mode::Hold => right_spammer.disable(),
                                    Mode::Toggle => right_spammer.disable(),
                                    Mode::Both => right_spammer.disable(),
                                    Mode::Always => unreachable!(),
                                }
                            }
                        }
                    }
                    _ => virtual_device.emit_silent(
                        event.event_type().0,
                        event.code(),
                        event.value(),
                    ),
                },
                2 if cli.scroll_changes_cps
                    && event.code() == 8
                    && (left_spammer.is_enabled() || right_spammer.is_enabled()) =>
                {
                    let stored_delay = if left_spammer.is_enabled() {
                        &state.left_click_delay_ns
                    } else if right_spammer.is_enabled() {
                        &state.right_click_delay_ns
                    } else {
                        // I'm 90% sure this is unreachable
                        // Yeah except if one manages to turn off a spammer between.
                        unreachable!(
                            "Expected left or right spammer enabled, but neither is enabled."
                        )
                    };

                    let delay = stored_delay.load(Ordering::Relaxed);
                    if event.value().is_positive() {
                        stored_delay.store(
                            (delay as f64 * (1.0 / cli.factor)) as u64 * event.value() as u64,
                            Ordering::Relaxed,
                        );
                    } else if event.value().is_negative() {
                        stored_delay.store(
                            (delay as f64 * cli.factor) as u64
                                * event.value().unsigned_abs() as u64,
                            Ordering::Relaxed,
                        );
                    }

                    if !(state.fast_enabled.load(Ordering::Relaxed) && left_spammer.is_enabled())
                        && Duration::from_nanos(stored_delay.load(Ordering::Relaxed)).as_secs_f64()
                            < cli.minimum_delay
                    {
                        stored_delay.store(
                            Duration::from_secs_f64(cli.minimum_delay).as_nanos() as u64,
                            Ordering::Relaxed,
                        );
                    }
                }
                // Basically just ignore these events if the scroll_changes_cps is enabled and used
                2 if cli.scroll_changes_cps
                    && [8, 11].contains(&event.code())
                    && (left_spammer.is_enabled() || right_spammer.is_enabled()) => {}
                _ => virtual_device.emit_silent(event.event_type().0, event.code(), event.value()),
            }
        }
    }
}
