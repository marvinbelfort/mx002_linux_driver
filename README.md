# MX002 Linux User-Space Driver

This README provides information about the Linux user-space driver for the graphics tablet sold in Brazil as the Multilaser MX002. The driver supports various rebranded versions of the same device.

## Supported Devices

This driver is intended for use with the following devices:
- Multilaser MX002
- VINSA 1060Plus
- Gotop Information Inc. [T501] Driver Inside Tablet
- 10moons G10/1060plus

To check if this driver is compatible with your device, you can run the following command:

```bash
lsusb | grep 08f2:6811
```

If your device is supported, the output should look similar to this:

```
Bus 001 Device 005: ID 08f2:6811 Gotop Information Inc. [T501] Driver Inside Tablet
```

## Features

- Utilizes the tablet's full area.
- All 12 physical buttons are functional.
- Both stylus buttons are operational.
- Supports about 700 levels of pressure sensitivity. While the device is marketed with 8192 levels, the hardware only reports 1024, of which roughly 700 are usable without risking damage to the stylus.

## Installation

To build and install the driver, follow these steps:

```bash
git clone git@github.com:marvinbelfort/mx002_linux_driver.git
cd mx002_linux_driver
cargo build --release
```

The driver will be located in `target/release/mx002`.

## Usage

To run the driver, you have two options:

1. **Run as Root:**

   ```bash
   sudo ./target/release/mx002
   ```

2. **Create a udev Rule:**

   For Arch Linux (this may vary by distribution):

   ```bash
   echo 'SUBSYSTEM=="usb", ATTRS{idVendor}=="08f2", ATTRS{idProduct}=="6811", TAG+="uaccess"' > /etc/udev/rules.d/75-mx002.rules
   sudo udevadm control --reload-rules
   sudo udevadm trigger
   ```

   This allows any user to run the driver without needing sudo.

## TODOs

- Enable more direct configuration of emitted key combinations.
- Implement mapping of the tablet area vs. multiple monitors.

## References

- [Tool that enables expanded mode for the tablet, by DigiMend](https://github.com/DIGImend/10moons-tools)
- [Learning about the possibility of creating user-space drivers](https://github.com/alex-s-v/10moons-driver)

This code is a combination of the two above, with some improvements.

