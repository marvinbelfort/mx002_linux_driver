mod virtual_device;
mod physical_device;
mod config;
use signal_hook::consts::signal::*;
use signal_hook::flag::register;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::u16;
use physical_device::PhysicalDevice;
use virtual_device::{DeviceDispatcher, RawDataReader};

// netoe1-mod Interface to change variables in config file.
// use crate::{ config::check_if_has_root_access, config::set_pen_strength, config::set_pen_threshold};

const VID: u16 = 0x08f2;
const PID: u16 = 0x6811;


fn main() {
    // netoe1-mod: Verifying if program is running as Sudo!
    config::check_if_has_root_access();

    let mut physical_device = PhysicalDevice::new(VID, PID);
    physical_device.init().set_full_mode();

    let mut data_reader = RawDataReader::new();
    let mut device_dispatcher = DeviceDispatcher::new();

    // netoe1-mod: Using input while executing, for tests.
    config::while_executing_input();
    println!("mx002-driver-log: The driver is running!");

    main_loop({
        || {
            if physical_device
                .read_device_responses(&mut data_reader.data)
                .is_ok()
            {
                device_dispatcher.dispatch(&data_reader);
                if device_dispatcher.syn().is_err() {
                    println!("mx002-driver-err:Error emitting SYN.");
                }
            }
        }
    });
}

fn main_loop(mut f: impl FnMut()) {
    let signals: Vec<i32> = vec![SIGINT, SIGTERM, SIGQUIT];
    let flag = Arc::new(AtomicBool::new(false));

    for signal in signals {
        register(signal, Arc::clone(&flag)).expect("Error registering interrupt signals.");
    }

    while !flag.load(Ordering::Relaxed) {
        f();
    }
    println!("mx002-driver-info: The driver is exiting.");
}