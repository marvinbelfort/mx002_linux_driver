#![allow(dead_code, unused)]

mod device_dispatcher;

use device_dispatcher::{DeviceDispatcher, RawDataReader};
use rusb::{
    devices, ConfigDescriptor, Device, DeviceHandle, Error as RusbError, GlobalContext,
    InterfaceDescriptor, TransferType,
};
use signal_hook::consts::signal::*;
use signal_hook::flag as signal_flag;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{u16, u8};

const VID: u16 = 0x08f2;
const PID: u16 = 0x6811;

fn main() -> Result<(), Box<dyn Error>> {
    let device = get_target_device()?;
    let configurations = get_configurations(&device);
    let interface_descriptors = get_hid_interface_descriptors(&configurations);

    let mut device_handler = device.open()?;
    device_handler.set_auto_detach_kernel_driver(true)?;

    let mut endpoint_address = 0;
    for interface_descriptor in interface_descriptors {
        device_handler.claim_interface(interface_descriptor.interface_number())?;
        for endpoint_descriptor in interface_descriptor.endpoint_descriptors() {
            if endpoint_descriptor.transfer_type() == TransferType::Interrupt
                && endpoint_descriptor.max_packet_size() == 64
            {
                endpoint_address = endpoint_descriptor.address();
            }
        }
    }

    set_report(&mut device_handler)?;

    let term = Arc::new(AtomicBool::new(false));
    signal_flag::register(SIGINT, Arc::clone(&term))?;
    signal_flag::register(SIGTERM, Arc::clone(&term))?;
    signal_flag::register(SIGQUIT, Arc::clone(&term))?;

    let mut data_reader = RawDataReader::new();
    let mut device_dispatcher = DeviceDispatcher::new();

    let mut buf = vec![0u8; 64];
    while !term.load(Ordering::Relaxed) {
        match device_handler.read_interrupt(endpoint_address, &mut data_reader.data, Duration::from_secs(3)) {
            Ok(bytes_read) => {
                device_dispatcher.dispatch(&data_reader);
            }
            Err(_e) => (),
        }
    }

    Ok(())
}

fn set_report(device_handler: &mut DeviceHandle<GlobalContext>) -> Result<(), RusbError> {
    const REPORTS: [[u8; 8]; 4] = [
        [0x08, 0x04, 0x1d, 0x01, 0xff, 0xff, 0x06, 0x2e],
        [0x08, 0x03, 0x00, 0xff, 0xf0, 0x00, 0xff, 0xf0],
        [0x08, 0x06, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00],
        [0x08, 0x03, 0x00, 0xff, 0xf0, 0x00, 0xff, 0xf0],
    ];

    for report in REPORTS.iter() {
        device_handler.write_control(0x21, 0x9, 0x0308, 2, report, Duration::from_millis(250))?;
    }

    Ok(())
}

fn is_target_device(device: &Device<GlobalContext>) -> bool {
    let device_descriptor = device.device_descriptor().unwrap();
    device_descriptor.vendor_id() == VID && device_descriptor.product_id() == PID
}

fn get_target_device() -> Result<Device<GlobalContext>, RusbError> {
    match devices()?.iter().find(is_target_device) {
        Some(device) => Ok(device),
        None => Err(RusbError::NoDevice),
    }
}

fn get_hid_interface_descriptors(
    config_descriptors: &[ConfigDescriptor],
) -> Vec<InterfaceDescriptor> {
    config_descriptors
        .iter()
        .flat_map(|config_descriptor| config_descriptor.interfaces())
        .flat_map(|interface| interface.descriptors())
        .filter(|interface_descriptor| {
            interface_descriptor.class_code() == rusb::constants::LIBUSB_CLASS_HID
        })
        .collect()
}

fn get_configurations(device: &Device<GlobalContext>) -> Vec<ConfigDescriptor> {
    let device_descriptor = device.device_descriptor().unwrap();
    (0..device_descriptor.num_configurations())
        .filter_map(|n| device.config_descriptor(n).ok())
        .collect()
}
