use signal_hook::consts::signal::*;
use signal_hook::flag::register;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::u16;

use mx002_lib::physical_device::PhysicalDevice;
use mx002_lib::virtual_device::{DeviceDispatcher, RawDataReader};

const VID: u16 = 0x08f2;
const PID: u16 = 0x6811;

fn main() {
    let mut physical_device = PhysicalDevice::new(VID, PID);
    physical_device.init().set_full_mode();

    let mut data_reader = RawDataReader::new();
    let mut device_dispatcher = DeviceDispatcher::new();

    main_loop({
        || match physical_device.read_device_responses(&mut data_reader.data) {
            Ok(_) => {
                device_dispatcher.dispatch(&data_reader);
            }
            Err(_) => (),
        }
    });
}

fn main_loop(mut f: impl FnMut() -> ()) {
    let signals: Vec<i32> = vec![SIGINT, SIGTERM, SIGQUIT];
    let flag = Arc::new(AtomicBool::new(false));

    for signal in signals {
        register(signal, Arc::clone(&flag)).expect("Error registering interrupt signals.");
    }

    while !flag.load(Ordering::Relaxed) {
        f();
    }
}
