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

To build and run the driver, follow these steps:

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
   sudo ./mx002
   ```

2. **Create a udev Rule:**

   For Arch Linux (this may vary by distribution):

   ```bash
   echo 'SUBSYSTEM=="usb", ATTRS{idVendor}=="08f2", ATTRS{idProduct}=="6811", TAG+="uaccess"' > /etc/udev/rules.d/99-mx002.rules
   sudo udevadm control --reload-rules
   sudo udevadm trigger
   ```

   This allows any user to run the driver without needing sudo.

## Multiple Monitors:

Show input devices with:
```bash
xinput

⎡ Virtual core pointer                    	id=2	[master pointer  (3)]
... 
⎜   ↳ virtual_tablet Pen (0)                  	id=10	[slave  pointer  (2)]
⎣ Virtual core keyboard                   	id=3	[master keyboard (2)]
...
    ↳ virtual_tablet                          	id=8	[slave  keyboard (3)]
    ↳ virtual_tablet                          	id=9	[slave  keyboard (3)]
```

If you can't see the entry "virtual_tablet Pen", move the pen a bit and run the command again.
Remember it's id (10 in the example)

List existing monitors:
```
xrandr | rg '\bconnected\b'

HDMI-0 connected primary 2560x1080+0+0 (normal left inverted right x axis y axis) 673mm x 284mm
DP-2 connected 1680x1050+2560+84 (normal left inverted right x axis y axis) 473mm x 296mm
```

So, to map the pen's movement only to monitor 2, use:

```bash
xinput map-to-output 10 "DP-2"
```



## TODOs

- Enable more direct configuration of emitted key combinations.
- Implement mapping of the tablet area vs. multiple monitors.
- Exhaust the SET REPORT HID protocol combinations to determine if any of them enable the advertised pressure of 8192. (but I really suspect that's all there is to it.)
- Add support for the 10 virtual buttons.

"## References

- [Tool that enables expanded mode for the tablet, by DigiMend](https://github.com/DIGImend/10moons-tools)
- [Learning about the possibility of creating user-space drivers](https://github.com/alex-s-v/10moons-driver)

This code is a combination of the two above, with some improvements.

