use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::sleep,
    time::Duration,
};

use clap::Parser;
use uinput_rs::{
    UInputUserDevice,
    key_codes::{
        BTN_EXTRA, BTN_LEFT, BTN_MIDDLE, BTN_MOUSE, BTN_RIGHT, BTN_SIDE, REL_HWHEEL,
        REL_HWHEEL_HI_RES, REL_WHEEL, REL_WHEEL_HI_RES, REL_X, REL_Y,
    },
    name_from_str,
};

use crate::{
    cli::{ClickMode, Mode},
    interface::{print_beginning, spawn_status_thread},
    spammer::Spammer,
};

mod cli;
mod interface;
mod spammer;

fn main() {
    let mut cli = cli::Cli::parse();
    if cli.click_mode == ClickMode::Cps {
        cli.left_click_delay = 1.0 / cli.left_click_delay;
        cli.right_click_delay = 1.0 / cli.right_click_delay;
        cli.fast_click_delay = 1.0 / cli.fast_click_delay;
    }
    if cli.debug {
        dbg!(&cli.mode);
    }

    let events = [
        BTN_MOUSE,
        BTN_RIGHT,
        REL_X,
        REL_Y,
        BTN_MIDDLE,
        BTN_SIDE,
        BTN_EXTRA,
        REL_WHEEL,
        REL_WHEEL_HI_RES,
        REL_HWHEEL,
        REL_HWHEEL_HI_RES,
    ];
    let device_info = UInputUserDevice {
        name: name_from_str("Rustclicker").unwrap(),
        ..Default::default()
    };
    let virtual_device = Arc::new(
        uinput_rs::Device::new_custom(&events, &device_info).expect("Error creating device."),
    );

    let left_enabled = Arc::new(AtomicBool::new(cli.enable_left));
    let right_enabled = Arc::new(AtomicBool::new(cli.enable_right));
    let fast_enabled = Arc::new(AtomicBool::new(cli.enable_fast));
    let click_counter = Arc::new(AtomicU64::new(0));
    let left_click_delay_ns = Arc::new(AtomicU64::new(
        Duration::from_secs_f64(cli.left_click_delay).as_nanos() as u64,
    ));
    let right_click_delay_ns = Arc::new(AtomicU64::new(
        Duration::from_secs_f64(cli.right_click_delay).as_nanos() as u64,
    ));

    let left_spammer = Spammer::new(
        virtual_device.clone(),
        BTN_LEFT,
        Duration::from_secs_f64(if cli.enable_fast {
            cli.fast_click_delay
        } else {
            cli.start_delay_left
        }),
        left_click_delay_ns.clone(),
        click_counter.clone(),
    );
    let right_spammer = Spammer::new(
        virtual_device.clone(),
        BTN_RIGHT,
        Duration::from_secs_f64(cli.start_delay_right),
        right_click_delay_ns.clone(),
        click_counter.clone(),
    );

    if cli.mode == Mode::Always {
        sleep(Duration::from_millis(100));
        print_beginning(
            &"No device needed",
            &cli.start_delay_left,
            &cli.start_delay_right,
            &cli.left_click_delay,
            &cli.right_click_delay,
            &cli.fast_click_delay,
        );
        if cli.enable_left {
            left_spammer.enable(cli.spammers);
        }
        if cli.enable_right {
            right_spammer.enable(cli.spammers);
        }
        loop {
            sleep(Duration::from_secs(60 * 60));
        }
    }

    let mut device = evdev::Device::open(
        cli.mouse_path
            .expect("Please give a mouse path is required if you use Hold or Toggle modes."),
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
    spawn_status_thread(
        click_counter.clone(),
        left_enabled.clone(),
        right_enabled.clone(),
        fast_enabled.clone(),
    );

    device.grab().expect("Error grabbing selected device.");

    while let Ok(events) = device.fetch_events() {
        for event in events {
            match event.event_type().0 {
                1 => match event.code() {
                    // left
                    272 => {
                        virtual_device.emit_key_code_silent(BTN_LEFT, event.value());

                        if event.value() == 1 {
                            click_counter.fetch_add(1, Ordering::Relaxed);
                            if left_enabled.load(Ordering::Relaxed) {
                                left_click_delay_ns.store(
                                    Duration::from_secs_f64(
                                        if fast_enabled.load(Ordering::Relaxed) {
                                            cli.fast_click_delay
                                        } else {
                                            cli.left_click_delay
                                        },
                                    )
                                    .as_nanos() as u64,
                                    Ordering::Relaxed,
                                );
                                match cli.mode {
                                    Mode::Hold => {
                                        if cli.debug {
                                            println!("Enabling spammer hold");
                                        }
                                        left_spammer.enable(cli.spammers);
                                    }
                                    Mode::Toggle => {
                                        if left_spammer.is_enabled() {
                                            if cli.debug {
                                                println!("Disabling spammer toggle");
                                            }
                                            left_spammer.disable();
                                        } else {
                                            if cli.debug {
                                                println!("Enabling spammer toggle");
                                            }
                                            left_spammer.enable(cli.spammers);
                                        }
                                    }
                                    Mode::Always => unreachable!(),
                                }
                            }
                        } else if event.value() == 0 && left_enabled.load(Ordering::Relaxed) {
                            match cli.mode {
                                Mode::Hold => {
                                    left_spammer.disable();
                                }
                                Mode::Toggle => {}
                                Mode::Always => unreachable!(),
                            }
                        }
                    }
                    // right
                    273 => {
                        virtual_device.emit_key_code_silent(BTN_RIGHT, event.value());

                        if event.value() == 1 {
                            click_counter.fetch_add(1, Ordering::Relaxed);
                            if right_enabled.load(Ordering::Relaxed) {
                                right_click_delay_ns.store(
                                    Duration::from_secs_f64(cli.right_click_delay).as_nanos()
                                        as u64,
                                    Ordering::Relaxed,
                                );
                                match cli.mode {
                                    Mode::Hold => {
                                        right_spammer.enable(cli.spammers);
                                    }
                                    Mode::Toggle => {
                                        if right_spammer.is_enabled() {
                                            right_spammer.disable();
                                        } else {
                                            right_spammer.enable(cli.spammers);
                                        }
                                    }
                                    Mode::Always => unreachable!(),
                                }
                            }
                        } else if event.value() == 0 && right_enabled.load(Ordering::Relaxed) {
                            match cli.mode {
                                Mode::Hold => {
                                    right_spammer.disable();
                                }
                                Mode::Toggle => {}
                                Mode::Always => unreachable!(),
                            }
                        }
                    }
                    // middle
                    274 => {
                        virtual_device.emit_key_code_silent(BTN_MIDDLE, event.value());
                        if event.value() == 1 {
                            fast_enabled.fetch_not(Ordering::Relaxed);
                            left_click_delay_ns.store(
                                Duration::from_secs_f64(if fast_enabled.load(Ordering::Relaxed) {
                                    cli.fast_click_delay
                                } else {
                                    cli.left_click_delay
                                })
                                .as_nanos() as u64,
                                Ordering::Relaxed,
                            );
                        }
                    }
                    // side
                    275 => {
                        if event.value() == 1 {
                            left_enabled.fetch_not(Ordering::Relaxed);
                            if !left_enabled.load(Ordering::Relaxed) {
                                left_spammer.disable();
                            }
                        }
                    }
                    // extra
                    276 => {
                        if event.value() == 1 {
                            right_enabled.fetch_not(Ordering::Relaxed);
                            if !right_enabled.load(Ordering::Relaxed) {
                                right_spammer.disable();
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
                        &left_click_delay_ns
                    } else if left_spammer.is_enabled() {
                        &right_click_delay_ns
                    } else {
                        // I'm 90% sure this is unreachable
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
                }
                _ => virtual_device.emit_silent(event.event_type().0, event.code(), event.value()),
            }
        }
    }
}
