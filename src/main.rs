use rusb::{
    devices, Device, DeviceDescriptor, Direction, Error as RusbError, GlobalContext, TransferType,
};

const VID: u16 = 0x08f2;
const PID: u16 = 0x6811;

fn main() -> Result<(), RusbError> {
    let device = get_device()?;
    let device_descriptor = device.device_descriptor().unwrap();
    let mut device_handle = device.open()?;

    if false {
        print_device_details(&device, &device_descriptor)?;
    }

    device_handle.set_auto_detach_kernel_driver(true)?;

    // Claim interfaces
    // Set Reports

    Ok(())
}

// fn name(arg: Type) -> RetType {
//     unimplemented!();
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

fn print_device_details(
    device: &Device<GlobalContext>,
    device_desc: &DeviceDescriptor,
) -> Result<(), RusbError> {
    for i in 0..device_desc.num_configurations() {
        println!("  Configuration {}", i + 1);
        for interface in device.config_descriptor(i)?.interfaces() {
            let interface_descriptors: Vec<_> = interface.descriptors().collect();
            for d in &interface_descriptors {
                let class_info =
                    ClassInfo::new(d.class_code(), d.sub_class_code(), d.protocol_code());

                println!(
                    "      Interface: {}{} ({}, {})",
                    interface.number(),
                    if interface_descriptors.len() > 1 {
                        format!("/{}", d.setting_number())
                    } else {
                        String::new()
                    },
                    class_info.formatted_class_name(),
                    class_info.formatted_subclass_protocol()
                );

                for e in d.endpoint_descriptors() {
                    println!(
                        "        Endpoint {:#04X}: {} {}",
                        e.address(),
                        match e.transfer_type() {
                            TransferType::Control => "CONTROL",
                            TransferType::Isochronous => "ISOCHRONOUS",
                            TransferType::Bulk => "BULK",
                            TransferType::Interrupt => "INTERRUPT",
                        },
                        match e.direction() {
                            Direction::In => "IN",
                            Direction::Out => "OUT",
                        }
                    )
                }
            }
        }
    }

    Ok(())
}

struct ClassInfo {
    class: u8,
    sub_class: u8,
    protocol: u8,
}

impl ClassInfo {
    fn new(class: u8, sub_class: u8, protocol: u8) -> Self {
        Self {
            class,
            sub_class,
            protocol,
        }
    }

    fn class_name(&self) -> Option<&str> {
        match self.class {
            0x00 => Some("Device"),
            0x01 => Some("Audio"),
            0x02 => Some("Communications and CDC Control"),
            0x03 => Some("Human Interface Device"),
            0x05 => Some("Physical"),
            0x06 => Some("Still Imaging"),
            0x07 => Some("Printer"),
            0x08 => Some("Mass Storage"),
            0x09 => Some("Hub"),
            0x0A => Some("CDC Data"),
            0x0B => Some("Smart Card"),
            0x0D => Some("Content Security"),
            0x0E => Some("Video"),
            0x0F => Some("Personal Healthcare"),
            0x10 => Some("Audio/Video"),
            0x11 => Some("Billboard"),
            0x12 => Some("USB Type-C Bridge"),
            0x3C => Some("I3C"),
            0xDC => Some("Diagnostic"),
            USB_DEVICE_CLASS_WIRELESS_CONTROLLER => Some("Wireless Controller"),
            0xEF => Some("Miscellaneous"),
            0xFE => Some("Application Specific"),
            0xFF => Some("Vendor Specific"),
            _ => None,
        }
    }

    fn protocol_name(&self) -> Option<&str> {
        match self.class {
            USB_DEVICE_CLASS_WIRELESS_CONTROLLER => match self.sub_class {
                0x01 => match self.protocol {
                    0x01 => Some("Bluetooth"),
                    0x02 => Some("UWB"),
                    0x03 => Some("Remote NDIS"),
                    0x04 => Some("Bluetooth AMP"),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
    }

    fn formatted_class_name(&self) -> String {
        self.class_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{:#04X}", self.class))
    }

    fn formatted_subclass_protocol(&self) -> String {
        format!(
            "{}/{}{}",
            self.sub_class,
            self.protocol,
            self.protocol_name()
                .map(|s| format!(" [{}]", s))
                .unwrap_or_default()
        )
    }
}

impl From<&DeviceDescriptor> for ClassInfo {
    fn from(value: &DeviceDescriptor) -> Self {
        Self::new(
            value.class_code(),
            value.sub_class_code(),
            value.protocol_code(),
        )
    }
}
