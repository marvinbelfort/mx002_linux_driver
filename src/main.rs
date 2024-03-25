//FIX: Removor no final
#![allow(dead_code)]
#![allow(unused_imports)]

use rusb::{
    devices, ConfigDescriptor, Device, DeviceHandle, Error as RusbError, GlobalContext, Interface,
};

use std::error::Error;
use std::{process::id, time::Duration};

use signal_hook::consts::signal::*;
use signal_hook::flag as signal_flag;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const VID: u16 = 0x08f2;
const PID: u16 = 0x6811;
const VAL: u16 = 0x0308;

const REQUEST_TYPE: u8 = 0x21;
const SET_REPORT: u8 = 9;

const REPORTS: [[u8; 8]; 4] = [
    [0x08, 0x04, 0x1d, 0x01, 0xff, 0xff, 0x06, 0x2e],
    [0x08, 0x03, 0x00, 0xff, 0xf0, 0x00, 0xff, 0xf0],
    [0x08, 0x06, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00],
    [0x08, 0x03, 0x00, 0xff, 0xf0, 0x00, 0xff, 0xf0],
];

fn main() -> Result<(), Box<dyn Error>> {
    let device = get_target_device()?;

    print!("Opening device\n");
    let mut device_handler = device.open()?;

    device_handler.set_auto_detach_kernel_driver(true)?;
    claim_and_set_reports(&mut device_handler)?;

    /* for i in 0..=2 {
        print!("Releasing iface {}\n", i);
        device_handler.release_interface(i)?;
    } */

    print!("Reseting Device \n");
    device_handler.reset()?;

    print!("");
    print!("Listing Endpoints:");
    get_interfaces(&get_configurations(&device))
        .iter()
        .for_each(|i| {
            print!("Interface {}\n", i.number());
            i.descriptors().for_each(|id| {
                id.endpoint_descriptors().for_each(|ed| {
                    print!("Endpoint address {}\n", ed.address());
                    print!("\t Endpoint max packet size {}\n", ed.max_packet_size());
                });
            });
        });

    // Aqui voiu fazer algumas chamadas que retornam Result<(), std::io::Error>, como integrar
    // a função main() para que ela atenda aos dois tipos de erro RusbError e Error?

    let term = Arc::new(AtomicBool::new(false));
    signal_flag::register(SIGINT, Arc::clone(&term))?;
    signal_flag::register(SIGTERM, Arc::clone(&term))?;
    signal_flag::register(SIGQUIT, Arc::clone(&term))?;
/*
    for i in 0..=2 {
        print!("Claiming Interface Again {} \n", i);
        device_handler.claim_interface(i)?
    }
*/
    let mut buf = [0u8; 64];
    while !term.load(Ordering::Relaxed) {
        // let result = device_handler.read_interrupt(133, &mut buf, Duration::from_secs(3));
        let result = device_handler.read_interrupt(131, &mut buf, Duration::from_secs(3));
        match result {
            Ok(bytes_read) => {
                println!("Bytes lidos: {}", bytes_read);
                println!("Dados: {:?}", &buf[..bytes_read]);
            }
            Err(e) => println!("Erro ao ler do endpoint: {:?}", e),
        }
    }

    Ok(())
}

fn realease_and_reattach(
    device_handler: &mut DeviceHandle<GlobalContext>,
) -> Result<(), RusbError> {
    //TODO: Descobrir se preciso ou não fazer o reattach do driver original
    for i in 0..=2 {
        print!("Releasing iface {}\n", i);
        device_handler.release_interface(i)?;
        print!("Reattach diver for iface {}\n", i);
        device_handler.attach_kernel_driver(i)?;
    }

    Ok(())
}

fn claim_and_set_reports(
    device_handler: &mut DeviceHandle<GlobalContext>,
) -> Result<(), RusbError> {
    //TODO: Descobrir se realmente preciso fazer claim de todas as interfaces.
    for i in 0..=2 {
        print!("Claiming Interface {} \n", i);
        device_handler.claim_interface(i)?
    }

    for report in REPORTS.iter() {
        print!("{:?}\n", report);
        device_handler.write_control(
            REQUEST_TYPE,
            SET_REPORT,
            VAL,
            2,
            report,
            Duration::from_millis(250),
        )?;
        print!("Write control successfull!\n");
    }
    Ok(())
}

fn is_target_device(device: &Device<GlobalContext>) -> bool {
    let device_descriptor = device.device_descriptor().unwrap();
    device_descriptor.vendor_id() == VID && device_descriptor.product_id() == PID
}

fn get_target_device() -> Result<Device<GlobalContext>, RusbError> {
    match devices()?.iter().find(|device| is_target_device(&device)) {
        Some(device) => Ok(device),
        None => Err(RusbError::NoDevice),
    }
}

fn get_interfaces(config_descriptors: &Vec<ConfigDescriptor>) -> Vec<Interface> {
    config_descriptors
        .iter()
        .map(|cd| cd.interfaces())
        .flatten()
        .collect()
}

fn get_configurations(device: &Device<GlobalContext>) -> Vec<ConfigDescriptor> {
    let device_descriptor = device.device_descriptor().unwrap();
    (0..device_descriptor.num_configurations())
        .filter_map(|n| device.config_descriptor(n).ok())
        .collect()
}
