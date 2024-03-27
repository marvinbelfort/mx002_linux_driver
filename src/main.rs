#![allow(dead_code)]
#![allow(unused_variables)]

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

enum PressState {
    Pressed = 0,
    Release = 1,
    Hold = 2,
    None = 3,
}

struct TabletButton {
    id: u8,
    state: PressState,
}

struct DeviceDispatcher {
    tablet_last_buttons: u16,
    pen_buttons: u8,
}

impl DeviceDispatcher {
    const X_AXIS_HIGH: usize = 1;
    const X_AXIS_LOW: usize = 2;
    const Y_AXIS_HIGH: usize = 3;
    const Y_AXIS_LOW: usize = 4;
    const Y_PRESSURE_HIGH: usize = 5;
    const Y_PRESSURE_LOW: usize = 6;
    const PEN_BUTTONS: usize = 9;
    const TABLET_BUTTONS_HIGH: usize = 12;
    const TABLET_BUTTONS_LOW: usize = 11;

    pub fn new() -> Self {
        DeviceDispatcher {
            tablet_last_buttons: 0,
            pen_buttons: 0,
        }
    }

    pub fn dispatch(&mut self, buffer: &[u8]) -> () {
        let binary_button_flags = Self::button_flags(
            buffer[Self::TABLET_BUTTONS_HIGH],
            buffer[Self::TABLET_BUTTONS_LOW],
        );

        let buttons = self.binary_to_buttons(binary_button_flags);
        self.tablet_last_buttons = binary_button_flags;

        // Mapeia botões pressionados para KEY_Events
        // Calcula boundaries e Emite X, Y
        // Calcula e emite Pen Pressure
    }

    fn button_flags(high: u8, low: u8) -> u16 {
        ((high | 0xcc) as u16) << 8 | low as u16
    }

    fn binary_to_buttons(&self, binary_button_flags: u16) -> Vec<TabletButton> {
        (0..14)
            .filter(|i| ![10, 11].contains(i))
            .map(|i| self.buttons_from_bits(i, binary_button_flags))
            .collect()
    }

    pub fn buttons_from_bits(&self, i: u8, binary_button_flags: u16) -> TabletButton {
        let mask = 1 << i;
        let is_pressed = (binary_button_flags & mask) == 0;
        let was_pressed = (self.tablet_last_buttons & mask) == 0;

        TabletButton {
            id: i,
            state: {
                match (was_pressed, is_pressed) {
                    (true, true) => PressState::Hold,
                    (false, true) => PressState::Pressed,
                    (false, false) => PressState::None,
                    (true, false) => PressState::Release,
                }
            },
        }
    }
}

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

    let mut device_dispatcher = DeviceDispatcher::new();

    let mut buf = vec![0u8; 64];
    while !term.load(Ordering::Relaxed) {
        match device_handler.read_interrupt(endpoint_address, &mut buf, Duration::from_secs(3)) {
            Ok(bytes_read) => {
                device_dispatcher.dispatch(&buf[..bytes_read]);
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
