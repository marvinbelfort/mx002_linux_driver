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

const VID: u16 = 0x08f2;
const PID: u16 = 0x6811;

fn main() -> Result<(), Box<dyn Error>> {
    let device = get_target_device()?;
    let configurations = get_configurations(&device);
    let interface_descriptors = get_interface_descriptors(&configurations);

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

    let mut buf = [0u8; 64];
    while !term.load(Ordering::Relaxed) {
        match device_handler.read_interrupt(endpoint_address, &mut buf, Duration::from_secs(3)) {
            Ok(bytes_read) => {
                print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
                println!("");
                let mut counter = 0;
                let mut position = 0;
                for byte in &buf[..bytes_read]{
                    if [1, 2, 3, 4, 5, 6, 9, 10, 11, 12, 13, 14].contains(&position){
                        print!("--- ");
                    } else {
                        print!("{:03} ", byte);
                    }
                    counter += 1;
                    if counter == 16 {
                        println!("");
                        counter = 0;
                    }
                    position += 1;
                }
                println!("---");
                println!("[01]: {:08} Axis X most" , &buf[01]);
                println!("[02]: {:08} Axis X last" , &buf[02]);
                println!("[03]: {:08} Axis Y most" , &buf[03]);
                println!("[13]: {:08} Axis Y most" , &buf[13]);
                println!("[04]: {:08} Axis Y last" , &buf[04]);
                println!("[14]: {:08} Axis Y last" , &buf[14]);
                println!("[05]: {:08} Pen Pressure most" , &buf[05]);
                println!("[06]: {:08} Pen Pressure last" , &buf[06]);
                println!("[09]: {:08b} Pen Btns", &buf[09]);
                println!("[10]: {:08b} Proximity", &buf[10]);
                println!("[11]: {:08b} Btns", &buf[11]);
                println!("[12]: {:08b} Btns", &buf[12]);
            }
            // Err(e) => println!("Erro ao ler do endpoint: {:?}", e),
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

fn get_interface_descriptors(config_descriptors: &[ConfigDescriptor]) -> Vec<InterfaceDescriptor> {
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
