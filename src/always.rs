use std::{thread::sleep, time::Duration};

use crate::{cli::Cli, interface::print_beginning, shared_state::SharedState};

pub fn run(cli: Cli, state: SharedState) -> ! {
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
        state.enable_spammer("left").unwrap();
    }
    if cli.enable_right {
        state.enable_spammer("right").unwrap();
    }
    loop {
        sleep(Duration::from_secs(60 * 60));
    }
}
