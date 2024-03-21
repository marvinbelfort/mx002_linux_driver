use rusb::{devices, Device, Error as RusbError, GlobalContext};

const VID: u16 = 0x08f2;
const PID: u16 = 0x6811;

fn main() -> Result<(), RusbError> {
    let device = get_device()?;
    println!("Dispositivo encontrado!");

    Ok(())
}

fn is_target_device(device: &Device<GlobalContext>) -> bool {
    let device_desc = device.device_descriptor().unwrap();
    device_desc.vendor_id() == VID && device_desc.product_id() == PID
}

fn get_device() -> Result<Device<GlobalContext>, RusbError> {
    match devices()?.iter().find(|device| is_target_device(&device)) {
        Some(device) => Ok(device),
        None => Err(RusbError::NoDevice),
    }
}
