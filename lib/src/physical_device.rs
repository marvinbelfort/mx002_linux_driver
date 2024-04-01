use rusb::{
    devices, ConfigDescriptor, Device, DeviceHandle, Error as RusbError, GlobalContext,
    InterfaceDescriptor, TransferType,
};

use std::time::Duration;

pub struct PhysicalDevice {
    endpoint_address: u8,
    device: Device<GlobalContext>,
    device_handle: DeviceHandle<GlobalContext>,
}

impl PhysicalDevice {
    pub fn new(vid: u16, pid: u16) -> Self {

        let device = Self::get_target_device(vid, pid).expect("Error finding device");

        PhysicalDevice {
            endpoint_address: 0,
            device_handle: device.open().expect("Error opening device"),
            device,
        }
    }

    pub fn init(&mut self) {
        self.device_handle.set_auto_detach_kernel_driver(true).expect("Error detaching old driver");

        let configurations = Self::get_configurations(&self.device);
        let interface_descriptors = Self::get_hid_interface_descriptors(&configurations);

        for interface_descriptor in interface_descriptors {
            self.device_handle
                .claim_interface(interface_descriptor.interface_number()).expect("Error claiming interface");
            for endpoint_descriptor in interface_descriptor.endpoint_descriptors() {
                if endpoint_descriptor.transfer_type() == TransferType::Interrupt
                    && endpoint_descriptor.max_packet_size() == 64
                {
                    self.endpoint_address = endpoint_descriptor.address();
                }
            }
        }
    }
   
    pub fn read(& self, buffer: &mut [u8]) -> Result<usize, RusbError> {
        self.device_handle.read_interrupt(self.endpoint_address, buffer, Duration::from_secs(3)) 
    }

    pub fn set_report(&mut self) -> Result<(), RusbError> {
        const REPORTS: [[u8; 8]; 4] = [
            [0x08, 0x04, 0x1d, 0x01, 0xff, 0xff, 0x06, 0x2e],
            [0x08, 0x03, 0x00, 0xff, 0xf0, 0x00, 0xff, 0xf0],
            [0x08, 0x06, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00],
            [0x08, 0x03, 0x00, 0xff, 0xf0, 0x00, 0xff, 0xf0],
        ];

        for report in REPORTS.iter() {
            self.device_handle.write_control(
                0x21,
                0x9,
                0x0308,
                2,
                report,
                Duration::from_millis(250),
            )?;
        }

        Ok(())
    }

    fn is_target_device(vid: u16, pid: u16, device: &Device<GlobalContext>) -> bool {
        let device_descriptor = device.device_descriptor().unwrap();
        device_descriptor.vendor_id() == vid && device_descriptor.product_id() == pid
    }

    fn get_target_device(vid: u16, pid: u16) -> Result<Device<GlobalContext>, RusbError> {
        match devices()?
            .iter()
            .find(|device| Self::is_target_device(vid, pid, device))
        {
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
}
