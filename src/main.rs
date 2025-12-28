use clap::{Parser, ValueEnum};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use std::{
    fmt::Display,
    io::{Write, stdout},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    },
    thread::{self, sleep},
    time::Duration,
};
use uinput_rs::{
    UInputUserDevice,
    key_codes::{
        BTN_EXTRA, BTN_MIDDLE, BTN_MOUSE, BTN_RIGHT, BTN_SIDE, REL_HWHEEL, REL_HWHEEL_HI_RES,
        REL_WHEEL, REL_WHEEL_HI_RES, REL_X, REL_Y,
    },
    name_from_str,
};

fn main() {
    let mut cli = Cli::parse();
    if cli.mode == Mode::Cps {
        cli.left_click_delay = 1.0 / cli.left_click_delay;
        cli.right_click_delay = 1.0 / cli.right_click_delay;
        cli.fast_left_click_delay = 1.0 / cli.fast_left_click_delay;
    }

    let mouse_left_down = Arc::new(AtomicBool::new(false));
    let mouse_right_down = Arc::new(AtomicBool::new(false));

    let left_enabled = Arc::new(AtomicBool::new(cli.enable_left));
    let right_enabled = Arc::new(AtomicBool::new(cli.enable_right));
    let fast_mode = Arc::new(AtomicBool::new(cli.enable_fast));

    let right_delay_ns = Arc::new(AtomicU64::new(
        Duration::from_secs_f32(cli.right_click_delay).as_nanos() as u64,
    ));
    let left_delay_ns = Arc::new(AtomicU64::new(
        Duration::from_secs_f32(if fast_mode.load(Ordering::Relaxed) {
            cli.left_click_delay
        } else {
            cli.fast_left_click_delay
        })
        .as_nanos() as u64,
    ));

    let clicks = Arc::new(AtomicU32::new(0));
    let cps = Arc::new(AtomicU32::new(0));

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
    let mut device = evdev::Device::open(cli.device_path).expect("Error opening device.");
    println!(
        "Selected device name: {}",
        device.name().unwrap_or_default()
    );
    println!(
        "Start delay:\nLeft {} | Right {}",
        cli.start_delay_left, cli.start_delay_right
    );
    println!(
        "Cps target:\n Left {} | Right {} | Fast {}",
        1.0 / cli.left_click_delay,
        1.0 / cli.right_click_delay,
        1.0 / cli.fast_left_click_delay
    );
    println!(
        "Click delay (seconds):\nLeft {} | Right {} | Fast {}",
        cli.left_click_delay, cli.right_click_delay, cli.fast_left_click_delay
    );

    device.grab().expect("Error grabbing selected device.");

    let clicks_copy = clicks.clone();
    let cps_copy = cps.clone();
    thread::spawn(move || {
        let mut buckets: AllocRingBuffer<u32> = AllocRingBuffer::new(100);
        loop {
            sleep(Duration::from_millis(10));
            let clicks = clicks_copy.load(Ordering::Relaxed);
            buckets.enqueue(clicks);
            clicks_copy.fetch_sub(clicks, Ordering::Relaxed);
            cps_copy.store(buckets.iter().sum::<u32>(), Ordering::Relaxed);
        }
    });

    let left_enabled_copy = left_enabled.clone();
    let right_enabled_copy = right_enabled.clone();
    let fast_mode_copy = fast_mode.clone();
    thread::spawn(move || {
        loop {
            sleep(Duration::from_millis(10));
            print_state(
                &left_enabled_copy,
                &right_enabled_copy,
                &fast_mode_copy,
                cps.load(Ordering::Relaxed),
            );
        }
    });

    loop {
        for event in device.fetch_events().expect("Error fetching events.") {
            match event.event_type().0 {
                1 => match event.code() {
                    272 => {
                        handle_result(virtual_device.emit_key_code(BTN_MOUSE, event.value()));
                        mouse_left_down.store(event.value() != 0, Ordering::Relaxed);

                        if event.value() == 1 {
                            clicks.fetch_add(1, Ordering::Relaxed);
                        }

                        if event.value() == 1 && left_enabled.load(Ordering::Relaxed) {
                            if fast_mode.load(Ordering::Relaxed) {
                                left_delay_ns.store(
                                    Duration::from_secs_f32(cli.fast_left_click_delay).as_nanos()
                                        as u64,
                                    Ordering::Relaxed,
                                );
                            } else {
                                left_delay_ns.store(
                                    Duration::from_secs_f32(cli.left_click_delay).as_nanos() as u64,
                                    Ordering::Relaxed,
                                );
                            }

                            for _ in 0..cli.spammers {
                                let device_copy = virtual_device.clone();
                                let left_enabled_copy = left_enabled.clone();
                                let left_down_copy = mouse_left_down.clone();
                                let cloned_clicks = clicks.clone();
                                let click_delay_ns = left_delay_ns.clone();
                                thread::spawn(move || {
                                    left_click_spam(
                                        device_copy,
                                        left_down_copy,
                                        cli.start_delay_left,
                                        left_enabled_copy,
                                        click_delay_ns,
                                        cloned_clicks,
                                    );
                                });
                            }
                        }
                    }
                    273 => {
                        handle_result(virtual_device.emit_key_code(BTN_RIGHT, event.value()));
                        mouse_right_down.store(event.value() != 0, Ordering::Relaxed);

                        if event.value() == 1 {
                            clicks.fetch_add(1, Ordering::Relaxed);
                        }

                        if event.value() == 1 && right_enabled.load(Ordering::Relaxed) {
                            right_delay_ns.store(
                                Duration::from_secs_f32(cli.right_click_delay).as_nanos() as u64,
                                Ordering::Relaxed,
                            );

                            for _ in 0..cli.spammers {
                                let device_copy = virtual_device.clone();
                                let right_enabled_copy = right_enabled.clone();
                                let right_down_copy = mouse_right_down.clone();
                                let cloned_clicks = clicks.clone();
                                let click_delay_ns = right_delay_ns.clone();
                                thread::spawn(move || {
                                    right_click_spam(
                                        device_copy,
                                        right_down_copy,
                                        right_enabled_copy,
                                        cli.start_delay_right,
                                        click_delay_ns,
                                        cloned_clicks,
                                    );
                                });
                            }
                        }
                    }
                    274 => {
                        handle_result(virtual_device.emit_key_code(BTN_MIDDLE, event.value()));
                        if event.value() == 1 {
                            fast_mode.fetch_not(Ordering::Relaxed);
                            if fast_mode.load(Ordering::Relaxed) {
                                left_delay_ns.store(
                                    Duration::from_secs_f32(cli.fast_left_click_delay).as_nanos()
                                        as u64,
                                    Ordering::Relaxed,
                                );
                            } else {
                                left_delay_ns.store(
                                    Duration::from_secs_f32(cli.left_click_delay).as_nanos() as u64,
                                    Ordering::Relaxed,
                                );
                            }
                        }
                    }
                    275 => {
                        if event.value() == 1 {
                            left_enabled.fetch_not(Ordering::Relaxed);
                        }
                    }
                    276 => {
                        if event.value() == 1 {
                            right_enabled.fetch_not(Ordering::Relaxed);
                        }
                    }
                    _ => {
                        handle_result(virtual_device.emit(
                            event.event_type().0,
                            event.code(),
                            event.value(),
                        ));
                    }
                },
                2 if [8, 11].contains(&event.code())
                    && ((right_enabled.load(Ordering::Relaxed)
                        || left_enabled.load(Ordering::Relaxed))
                        && (mouse_right_down.load(Ordering::Relaxed)
                            || mouse_left_down.load(Ordering::Relaxed))) =>
                {
                    let stored_delay = if left_enabled.load(Ordering::Relaxed)
                        && mouse_left_down.load(Ordering::Relaxed)
                    {
                        &left_delay_ns
                    } else if right_enabled.load(Ordering::Relaxed)
                        && mouse_right_down.load(Ordering::Relaxed)
                    {
                        &right_delay_ns
                    } else {
                        handle_result(virtual_device.emit(
                            event.event_type().0,
                            event.code(),
                            event.value(),
                        ));
                        continue;
                    };

                    if event.code() == 11 {
                        continue;
                    }

                    if event.value().is_positive() {
                        let delay = stored_delay.load(Ordering::Relaxed);
                        stored_delay.store(
                            (delay as f64 * (1.0 / cli.factor_scroll)) as u64
                                * event.value() as u64,
                            Ordering::Relaxed,
                        );
                    } else if event.value().is_negative() {
                        let delay = stored_delay.load(Ordering::Relaxed);
                        stored_delay.store(
                            (delay as f64 * cli.factor_scroll) as u64
                                * event.value().unsigned_abs() as u64,
                            Ordering::Relaxed,
                        );
                    }
                    // println!("Speed is now {}", stored_delay.load(Ordering::Relaxed));
                }
                _ => {
                    handle_result(virtual_device.emit(
                        event.event_type().0,
                        event.code(),
                        event.value(),
                    ));
                }
            };
        }
    }
}

fn right_click_spam(
    virtual_device: Arc<uinput_rs::Device>,
    mouse_right_down: Arc<AtomicBool>,
    right_enabled: Arc<AtomicBool>,
    start_delay_right: f32,
    click_delay_ns: Arc<AtomicU64>,
    clicks: Arc<AtomicU32>,
) {
    sleep(Duration::from_secs_f32(start_delay_right));

    while mouse_right_down.load(Ordering::Relaxed) && right_enabled.load(Ordering::Relaxed) {
        handle_result(virtual_device.emit_key_code(BTN_RIGHT, 1));
        handle_result(virtual_device.sync());
        sleep(Duration::from_nanos(click_delay_ns.load(Ordering::Relaxed)));
        handle_result(virtual_device.emit_key_code(BTN_RIGHT, 0));
        handle_result(virtual_device.sync());
        clicks.fetch_add(1, Ordering::Relaxed);
    }
}

fn left_click_spam(
    virtual_device: Arc<uinput_rs::Device>,
    mouse_left_down: Arc<AtomicBool>,
    start_delay_left: f32,
    left_enabled: Arc<AtomicBool>,
    click_delay_ns: Arc<AtomicU64>,
    clicks: Arc<AtomicU32>,
) {
    sleep(Duration::from_secs_f32(start_delay_left));

    while mouse_left_down.load(Ordering::Relaxed) && left_enabled.load(Ordering::Relaxed) {
        handle_result(virtual_device.emit_key_code(BTN_MOUSE, 1));
        handle_result(virtual_device.sync());
        sleep(Duration::from_nanos(click_delay_ns.load(Ordering::Relaxed)));
        handle_result(virtual_device.emit_key_code(BTN_MOUSE, 0));
        handle_result(virtual_device.sync());
        clicks.fetch_add(1, Ordering::Relaxed);
    }
}

/// relaxed way to handle a result
fn handle_result(result: Result<(), std::io::Error>) {
    if result.is_err() {
        println!("Error firing an event: {}", result.unwrap_err());
    }
}

fn print_state<T: Display>(
    left_enabled: &Arc<AtomicBool>,
    right_enabled: &Arc<AtomicBool>,
    fast_mode: &Arc<AtomicBool>,
    cps: T,
) {
    let longest = format!(
        "left: false | right: false | fast: false | cps: {}",
        u32::MAX
    )
    .len();
    print!("\r{}", " ".repeat(longest));
    print!(
        "\rLeft: {} | Right: {} | Fast: {} | cps: {cps}",
        left_enabled.load(Ordering::Relaxed),
        right_enabled.load(Ordering::Relaxed),
        fast_mode.load(Ordering::Relaxed),
    );
    handle_result(stdout().flush());
}

#[derive(Parser)]
#[command(version, about = "Rust auto clicker. (Delays are in seconds)", long_about = None)]
struct Cli {
    /// path to /dev/input/event*
    device_path: String,

    /// Normal left click delay
    #[arg(short, long, default_value_t = 0.05)]
    left_click_delay: f32,

    /// Fast left click delay
    #[arg(short, long, default_value_t = 0.02)]
    fast_left_click_delay: f32,

    /// Right click delay
    #[arg(short, long, default_value_t = 0.03)]
    right_click_delay: f32,

    /// In what format are the input delays given.
    /// Use the same delay parameters in any case.
    /// A delay under 1000 nanoseconds will not scale linearly and will cap at around 17k cps.
    /// You can get faster by putting the click delay to zero.
    #[arg(short, long, value_enum, default_value_t = Mode::Delay)]
    mode: Mode,

    /// Whether to start with left click enabled.
    #[arg(long)]
    enable_left: bool,

    /// Whether to start with right click enabled.
    #[arg(long)]
    enable_right: bool,

    /// Whether to start with fast left click enabled.
    #[arg(long)]
    enable_fast: bool,

    /// Delay before to start clicking
    #[arg(long, default_value_t = 0.1)]
    start_delay_left: f32,

    /// Delay before to start clicking
    #[arg(long, default_value_t = 0.1)]
    start_delay_right: f32,

    /// Change cps when autoclicking by scrolling.
    /// Left click takes priority if both are active.
    #[arg(short, long)]
    scroll_change_cps: bool,

    /// Factor of how much the scroll changes the delay.
    #[arg(short, long, default_value_t = 1.1)]
    factor_scroll: f64,

    /// If enabled, instantly starts to spam enabled keys before killed.
    #[arg(short, long)]
    always: bool,

    /// How many spammers to spawn when activating the auto clicker.
    /// This is like a multiplier for cps.
    /// I will not take any responsibility for changing this parameter.
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..))]
    spammers: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    /// Input the delay between clicks.
    Delay,
    /// Input the target cps instead of the delay between clicks.
    /// This is an afterthought feature.
    /// Don't expect it to be perfect.
    Cps,
}
