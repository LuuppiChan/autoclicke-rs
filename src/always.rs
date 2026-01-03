use std::{thread::sleep, time::Duration};

use crate::{cli::Cli, interface::print_beginning, spammer::Spammer};

pub fn run(cli: Cli, left_spammer: Spammer, right_spammer: Spammer) -> ! {
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
