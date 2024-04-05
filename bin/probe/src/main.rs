#![allow(unused)]

use signal_hook::consts::signal::*;
use signal_hook::flag as signal_flag;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::u16;

use mx002_lib::physical_device::PhysicalDevice;
use mx002_lib::virtual_device::{DeviceDispatcher, RawDataReader};

const VID: u16 = 0x08f2;
const PID: u16 = 0x6811;

const DEFAULT: [[u8; 8]; 4] = [
    [0x08, 0x04, 0x1d, 0x01, 0xff, 0xff, 0x06, 0x2e],
    [0x08, 0x03, 0x00, 0xff, 0xf0, 0x00, 0xff, 0xf0],
    [0x08, 0x06, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00],
    [0x08, 0x03, 0x00, 0xff, 0xf0, 0x00, 0xff, 0xf0],
];

//Sim
const A1: [[u8; 8]; 3] = [
    [0x08, 0x04, 0x1d, 0x01, 0xff, 0xff, 0x06, 0x2e],
    [0x08, 0x06, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00],
    [0x08, 0x03, 0x00, 0xff, 0xf0, 0x00, 0xff, 0xf0],
];

// Sim
const A2: [[u8; 8]; 3] = [
    [0x08, 0x03, 0x00, 0xff, 0xf0, 0x00, 0xff, 0xf0],
    [0x08, 0x04, 0x1d, 0x01, 0xff, 0xff, 0x06, 0x2e],
    [0x08, 0x06, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00],
];

//Sim 
const MIN: [[u8; 8]; 1] = [
    [0x08, 0x03, 0x00, 0xff, 0xf0, 0x00, 0xff, 0xf0],
];

const T1: [[u8; 8]; 2] = [
    [0x08, 0x04, 0x1d, 0x01, 0xff, 0xff, 0x06, 0x2e],
    [0x08, 0x03, 0x00, 0xff, 0xf0, 0x00, 0xff, 0xf0],
];

const T2: [[u8; 8]; 2] = [
    [0x08, 0x06, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00],
    [0x08, 0x03, 0x00, 0xff, 0xf0, 0x00, 0xff, 0xf0],
];


// Não
const T4: [[u8; 8]; 1] = [
    [0x08, 0x04, 0x1d, 0x01, 0xff, 0xff, 0x06, 0x2e],
];

//Não parece funcionar sozinho
const T5: [[u8; 8]; 1] = [
    [0x08, 0x06, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00],
];

fn main() -> Result<(), Box<dyn Error>> {
    let mut physical_device = PhysicalDevice::new(VID, PID);
    physical_device.init();
    physical_device.reset();
    physical_device.set_report(&A2.iter().map(|r| &r[..]).collect::<Vec<&[u8]>>());

    let term = Arc::new(AtomicBool::new(false));
    signal_flag::register(SIGINT, Arc::clone(&term))?;
    signal_flag::register(SIGTERM, Arc::clone(&term))?;
    signal_flag::register(SIGQUIT, Arc::clone(&term))?;

    let mut data_reader = RawDataReader::new();
    let mut device_dispatcher = DeviceDispatcher::new();

    while !term.load(Ordering::Relaxed) {
        let mut buffer = vec![0u8; 64];
        match physical_device.read_device_responses(&mut buffer) {
            Ok(bytes_read) => {
                let mut counter = 0;
                print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
                for byte in &buffer[..bytes_read] {
                    print!("{:02x} ", byte);
                    counter += 1;
                    if counter == 16 {
                        counter = 0;
                    }
                }
            }
            Err(_e) => (),
        }
    }

    Ok(())
}
