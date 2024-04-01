use signal_hook::consts::signal::*;
use signal_hook::flag as signal_flag;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::u16;

use mx002_lib::virtual_device::{DeviceDispatcher, RawDataReader};
use mx002_lib::physical_device::PhysicalDevice;

const VID: u16 = 0x08f2;
const PID: u16 = 0x6811;

fn main() -> Result<(), Box<dyn Error>> {

    let mut physical_device = PhysicalDevice::new(VID, PID);
    physical_device.init();
    physical_device.set_report().expect("Error setting report");

    let term = Arc::new(AtomicBool::new(false));
    signal_flag::register(SIGINT, Arc::clone(&term))?;
    signal_flag::register(SIGTERM, Arc::clone(&term))?;
    signal_flag::register(SIGQUIT, Arc::clone(&term))?;

    let mut data_reader = RawDataReader::new();
    let mut device_dispatcher = DeviceDispatcher::new();

    while !term.load(Ordering::Relaxed) {
        match physical_device.read( &mut data_reader.data) {
            Ok(_bytes_read) => {
                device_dispatcher.dispatch(&data_reader);
            }
            Err(_e) => (),
        }
    }

    Ok(())
}

