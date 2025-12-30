use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(
    version,
    about = "An advanced autoclicker written in rust. (Delays are in seconds)",
    long_about = "This autoclicker can either work when holding down a mouse key, always when on and with mouse keys togging it on and off."
)]
pub struct Cli {
    /// Will autoclick based on this mouse.
    pub mouse_path: Option<String>,

    /// Delay between clicks (default) or target cps.
    #[arg(short, long, default_value_t = 0.05)]
    pub left_click_delay: f64,

    /// Delay between clicks (default) or target cps.
    #[arg(short, long, default_value_t = 0.05)]
    pub right_click_delay: f64,

    /// Fast mode can be enabled for left clicking.
    /// Delay between clicks (default) or target cps.
    #[arg(short, long, default_value_t = 0.02)]
    pub fast_click_delay: f64,

    /// In what format are the input delays given.
    /// Use the same delay parameters in any case.
    /// A delay under 1000 nanoseconds will not scale linearly and will cap at around 17k cps.
    /// You can get faster by putting the click delay to zero.
    #[arg(short, long, value_enum, default_value_t = ClickMode::Delay)]
    pub click_mode: ClickMode,

    /// Program operation mode.
    #[arg(short, long, value_enum, default_value_t = Mode::Hold)]
    pub mode: Mode,

    /// Start left click enabled.
    #[arg(long)]
    pub enable_left: bool,

    /// Start right click enabled.
    #[arg(long)]
    pub enable_right: bool,

    /// Start fast mode enabled.
    #[arg(long)]
    pub enable_fast: bool,

    /// Delay before to start left clicking.
    #[arg(long, default_value_t = 0.1)]
    pub start_delay_left: f64,

    /// Delay before to start right clicking.
    #[arg(long, default_value_t = 0.1)]
    pub start_delay_right: f64,

    /// Change cps when autoclicking by scrolling.
    /// Left click takes priority when you're clicking with both.
    /// Will reset after stopping clicking.
    /// Does not work with always mode.
    #[arg(short, long)]
    pub scroll_changes_cps: bool,

    /// Factor of how much the scroll changes the delay.
    #[arg(long, default_value_t = 1.1)]
    pub factor: f64,

    /// Minimum delay allowed when scrolling.
    /// If fast mode is enabled, this is ignored.
    /// If input mode is cps this will be the max cps.
    #[arg(long, default_value_t = 0.0)]
    pub minimum_delay: f64,

    /// How many spammers to spawn when activating the autoclicker.
    /// This is like a multiplier for cps.
    /// I will not take any responsibility for changing this parameter.
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..))]
    pub spammers: u8,

    /// Print useful information for debugging.
    /// (Not fully ready)
    #[arg(short, long)]
    pub debug: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ClickMode {
    /// Delay between clicks.
    Delay,
    /// Target cps.
    Cps,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Mode {
    /// Clicks when holding the button down.
    Hold,
    /// Toggles clicking so you don't have to hold anything to click.
    Toggle,
    /// Instantly starts to spam enabled keys before the program is killed.
    Always,
}
