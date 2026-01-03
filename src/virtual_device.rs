use uinput_rs::{
    Device, UInputUserDevice,
    key_codes::{
        BTN_EXTRA, BTN_LEFT, BTN_MIDDLE, BTN_RIGHT, BTN_SIDE, REL_HWHEEL, REL_HWHEEL_HI_RES,
        REL_WHEEL, REL_WHEEL_HI_RES, REL_X, REL_Y,
    },
    name_from_str,
};

pub fn get() -> Device {
    let events = [
        BTN_LEFT,
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
    Device::new_custom(&events, &device_info).expect("Error creating device.")
}
