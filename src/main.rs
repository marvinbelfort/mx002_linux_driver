use rusb::{
    devices, ConfigDescriptor, Device, DeviceDescriptor, Direction, Error as RusbError,
    GlobalContext, TransferType, Interface,
};

const VID: u16 = 0x08f2;
const PID: u16 = 0x6811;

fn main() -> Result<(), RusbError> {
    let device = get_device()?;
    let mut device_handle = device.open()?;

    device_handle.set_auto_detach_kernel_driver(true)?;

    // Claim interfaces
    for i in 0..=2 {
        print!("Claiming Interface {} \n", i);
        device_handle.claim_interface(i)?
    }

    // Set Reports

    Ok(())
}

// fn get_interfaces(config_descriptors: &Vec<ConfigDescriptor>) -> Vec<Interface> {
//     config_descriptors.iter().map()
// }

// fn get_configurations(device: &Device<GlobalContext>) -> Vec<ConfigDescriptor> {
//     let device_descriptor = device.device_descriptor().unwrap();
//     (0..device_descriptor.num_configurations())
//         .filter_map(|n| device.config_descriptor(n).ok())
//         .collect()
// }

fn is_target_device(device: &Device<GlobalContext>) -> bool {
    let device_descriptor = device.device_descriptor().unwrap();
    device_descriptor.vendor_id() == VID && device_descriptor.product_id() == PID
}

fn get_device() -> Result<Device<GlobalContext>, RusbError> {
    match devices()?.iter().find(|device| is_target_device(&device)) {
        Some(device) => Ok(device),
        None => Err(RusbError::NoDevice),
    }
}
