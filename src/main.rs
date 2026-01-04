use std::{
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

use uinput_rs::key_codes::{BTN_LEFT, BTN_MIDDLE, BTN_RIGHT};

use crate::{
    cli::Mode,
    handle_click::handle_click,
    interface::{print_beginning, spawn_cps_calculator, spawn_status_thread},
    spammer::Spammer,
};

mod always;
mod cli;
mod conversions;
mod handle_click;
mod interface;
mod shared_state;
mod spammer;
mod virtual_device;

fn main() {
    let cli = cli::parse();
    let virtual_device = Arc::new(virtual_device::get());

    let state = shared_state::init(&cli);
    let mut last_left_click = Instant::now();
    let mut last_right_click = Instant::now();

    let left_spammer = Spammer::new(virtual_device.clone())
        .click_counter(state.click_counter.clone())
        .key(BTN_LEFT)
        .click_delay(Duration::from_secs_f64(cli.left_click_delay))
        .start_delay(Duration::from_secs_f64(cli.start_delay_left))
        .quantity(cli.spammers)
        .randomize(cli.randomize)
        .deviation(cli.deviation);
    let right_spammer = Spammer::new(virtual_device.clone())
        .click_counter(state.click_counter.clone())
        .key(BTN_RIGHT)
        .click_delay(Duration::from_secs_f64(cli.right_click_delay))
        .start_delay(Duration::from_secs_f64(cli.start_delay_right))
        .quantity(cli.spammers)
        .randomize(cli.randomize)
        .deviation(cli.deviation);
    state.add_spammer("left", left_spammer);
    state.add_spammer("right", right_spammer);

    if cli.mode == Mode::Always {
        always::run(cli, state);
    } else if cli.mode == Mode::Toggle {
        if cli.enable_fast {
            state
                .get_spammer("left")
                .unwrap()
                .lock()
                .unwrap()
                .set_click_delay_f(cli.fast_click_delay);
        }
        if cli.enable_left {
            state.enable_spammer("left").unwrap();
        }
        if cli.enable_right {
            state.enable_spammer("right").unwrap();
        }
    }

    let mut device = evdev::Device::open(
        cli.mouse_path
            .clone()
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
    spawn_cps_calculator(state.clone(), cli.update_delay);
    spawn_status_thread(state.clone());

    device.grab().expect("Error grabbing selected device.");

    while let Ok(events) = device.fetch_events() {
        for event in events {
            match event.event_type().0 {
                1 => match event.code() {
                    // left
                    272 => {
                        virtual_device.emit_key_code_silent(BTN_LEFT, event.value());
                        handle_click(
                            &event,
                            &state,
                            &cli,
                            "left",
                            &state.left_enabled,
                            if state.fast_enabled.load(Ordering::Relaxed) {
                                cli.fast_click_delay
                            } else {
                                cli.left_click_delay
                            },
                            cli.start_delay_left,
                            &mut last_left_click,
                        );
                    }
                    // right
                    273 => {
                        virtual_device.emit_key_code_silent(BTN_RIGHT, event.value());
                        handle_click(
                            &event,
                            &state,
                            &cli,
                            "right",
                            &state.right_enabled,
                            cli.right_click_delay,
                            cli.start_delay_right,
                            &mut last_right_click,
                        );
                    }
                    // middle
                    274 => {
                        virtual_device.emit_key_code_silent(BTN_MIDDLE, event.value());
                        if event.value() == 1 && state.left_enabled.load(Ordering::Relaxed) {
                            state.fast_enabled.fetch_not(Ordering::Relaxed);
                            state
                                .get_spammer("left")
                                .unwrap()
                                .lock()
                                .unwrap()
                                .set_click_delay_f(if state.fast_enabled.load(Ordering::Relaxed) {
                                    cli.fast_click_delay
                                } else {
                                    cli.left_click_delay
                                });
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
                                        state
                                            .get_spammer("left")
                                            .unwrap()
                                            .lock()
                                            .unwrap()
                                            .set_click_delay_f(
                                                if state.fast_enabled.load(Ordering::Relaxed) {
                                                    cli.fast_click_delay
                                                } else {
                                                    cli.left_click_delay
                                                },
                                            );
                                        state.enable_spammer("left").unwrap();
                                    }
                                    Mode::Both => (),
                                    Mode::Always => unreachable!(),
                                }
                            } else {
                                match cli.mode {
                                    Mode::Hold => state.disable_spammer("left"),
                                    Mode::Toggle => state.disable_spammer("left"),
                                    Mode::Both => state.disable_spammer("left"),
                                    Mode::Always => unreachable!(),
                                }
                                .unwrap();
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
                                        state
                                            .get_spammer("right")
                                            .unwrap()
                                            .lock()
                                            .unwrap()
                                            .set_click_delay_f(cli.right_click_delay);
                                        state.enable_spammer("right").unwrap();
                                    }
                                    Mode::Both => (),
                                    Mode::Always => unreachable!(),
                                }
                            } else {
                                match cli.mode {
                                    Mode::Hold => state.disable_spammer("right"),
                                    Mode::Toggle => state.disable_spammer("right"),
                                    Mode::Both => state.disable_spammer("right"),
                                    Mode::Always => unreachable!(),
                                }
                                .unwrap();
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
                    && (state.is_enabled_spammer("left").unwrap()
                        || state.is_enabled_spammer("right").unwrap()) =>
                {
                    let stored_delay = if state.is_enabled_spammer("left").unwrap() {
                        state
                            .get_spammer("left")
                            .unwrap()
                            .lock()
                            .unwrap()
                            .click_delay_ns
                            .clone()
                    } else if state.is_enabled_spammer("right").unwrap() {
                        state
                            .get_spammer("right")
                            .unwrap()
                            .lock()
                            .unwrap()
                            .click_delay_ns
                            .clone()
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

                    if !(state.fast_enabled.load(Ordering::Relaxed)
                        && state.is_enabled_spammer("left").unwrap())
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
                    && (state.is_enabled_spammer("left").unwrap()
                        || state.is_enabled_spammer("right").unwrap()) => {}
                _ => virtual_device.emit_silent(event.event_type().0, event.code(), event.value()),
            }
        }
    }
}
