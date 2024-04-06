use std::io::Error;
use std::{collections::HashMap, u16};

use evdev_rs::enums::{EventCode, EV_ABS, EV_KEY, EV_SYN};
use evdev_rs::{
    AbsInfo, DeviceWrapper, EnableCodeData, InputEvent, TimeVal, UInputDevice, UninitDevice,
};

#[derive(Default)]
pub struct RawDataReader {
    pub data: Vec<u8>,
}

impl RawDataReader {
    const X_AXIS_HIGH: usize = 1;
    const X_AXIS_LOW: usize = 2;
    const Y_AXIS_HIGH: usize = 3;
    const Y_AXIS_LOW: usize = 4;
    const PRESSURE_HIGH: usize = 5;
    const PRESSURE_LOW: usize = 6;
    const PEN_BUTTONS: usize = 9;
    const TABLET_BUTTONS_HIGH: usize = 12;
    const TABLET_BUTTONS_LOW: usize = 11;

    pub fn new() -> Self {
        RawDataReader {
            data: vec![0u8; 64],
        }
    }

    fn u16_from_2_u8(&self, high: u8, low: u8) -> u16 {
        (high as u16) << 8 | low as u16
    }

    fn x_axis(&self) -> i32 {
        self.u16_from_2_u8(self.data[Self::X_AXIS_HIGH], self.data[Self::X_AXIS_LOW]) as i32
    }

    fn y_axis(&self) -> i32 {
        self.u16_from_2_u8(self.data[Self::Y_AXIS_HIGH], self.data[Self::Y_AXIS_LOW]) as i32
    }

    fn pressure(&self) -> i32 {
        self.u16_from_2_u8(
            self.data[Self::PRESSURE_HIGH],
            self.data[Self::PRESSURE_LOW],
        ) as i32
    }

    fn tablet_buttons_as_binary_flags(&self) -> u16 {
        self.u16_from_2_u8(
            self.data[Self::TABLET_BUTTONS_HIGH],
            self.data[Self::TABLET_BUTTONS_LOW],
        ) | (0xcc << 8)
    }

    fn pen_buttons(&self) -> u8 {
        self.data[Self::PEN_BUTTONS]
    }
}

pub struct VirtualDevice {
    uinput_device: UInputDevice,
}

impl VirtualDevice {
    pub fn emit(&self, event_code: EventCode, value: i32) -> Result<(), Error> {
        self.uinput_device.write_event(&InputEvent {
            time: TimeVal {
                tv_sec: 0,
                tv_usec: 0,
            },
            event_code,
            value,
        })?;
        Ok(())
    }

    pub fn syn(&self) -> Result<(), Error> {
        self.emit(EventCode::EV_SYN(EV_SYN::SYN_REPORT), 0)?;
        Ok(())
    }
}

pub struct VirtualDeviceBuilder {
    uninit_device: UninitDevice,
}

impl VirtualDeviceBuilder {
    pub fn new(name: &str) -> Option<Self> {
        if let Some(uninit_device) = UninitDevice::new() {
            uninit_device.set_name(name);
            return Some(VirtualDeviceBuilder { uninit_device });
        };
        None
    }

    pub fn enable_keys(&mut self, keys: &[EV_KEY]) -> Result<&mut Self, Error> {
        for &key in keys {
            if self.uninit_device.enable(EventCode::EV_KEY(key)).is_err() {
                println!("Error enabling key.");
            }
        }
        Ok(self)
    }

    pub fn enable_abs(&mut self, ev_abs: EV_ABS, abs_info: AbsInfo) -> Result<&mut Self, Error> {
        let enabled_code_data_x = EnableCodeData::AbsInfo(abs_info);
        self.uninit_device
            .enable_event_code(&EventCode::EV_ABS(ev_abs), Some(enabled_code_data_x))?;
        Ok(self)
    }

    pub fn build(&mut self) -> Result<VirtualDevice, Error> {
        self.uninit_device
            .enable(EventCode::EV_SYN(EV_SYN::SYN_REPORT))?;
        let uinput_device = UInputDevice::create_from_device(&self.uninit_device)?;
        Ok(VirtualDevice { uinput_device })
    }
}

pub struct DeviceDispatcher {
    tablet_last_raw_pressed_buttons: u16,
    pen_last_raw_pressed_button: u8,
    tablet_button_id_to_key_code_map: HashMap<u8, Vec<EV_KEY>>,
    pen_button_id_to_key_code_map: HashMap<u8, Vec<EV_KEY>>,
    virtual_pen: VirtualDevice,
    virtual_keyboard: VirtualDevice,
    was_touching: bool,
}

impl Default for DeviceDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceDispatcher {
    const PRESSED: i32 = 1;
    const RELEASED: i32 = 0;
    const HOLD: i32 = 2;

    pub fn new() -> Self {
        let default_tablet_button_id_to_key_code_map: HashMap<u8, Vec<EV_KEY>> = [
            (0, vec![EV_KEY::KEY_TAB]),
            (1, vec![EV_KEY::KEY_SPACE]),
            (2, vec![EV_KEY::KEY_LEFTALT]),
            (3, vec![EV_KEY::KEY_LEFTCTRL]),
            (4, vec![EV_KEY::KEY_SCROLLDOWN]),
            (5, vec![EV_KEY::KEY_SCROLLUP]),
            (6, vec![EV_KEY::KEY_LEFTBRACE]),
            (7, vec![EV_KEY::KEY_LEFTCTRL, EV_KEY::KEY_KPMINUS]),
            (8, vec![EV_KEY::KEY_KPPLUS]),
            (9, vec![EV_KEY::KEY_E]),
            //10 This code is not emitted by physical device
            //11 This code is not emitted by physical device
            (12, vec![EV_KEY::KEY_B]),
            (13, vec![EV_KEY::KEY_RIGHTBRACE]),
        ]
        .iter()
        .cloned()
        .collect();

        let default_pen_button_id_to_key_code_map: HashMap<u8, Vec<EV_KEY>> = [
            (4, vec![EV_KEY::BTN_STYLUS]),
            (6, vec![EV_KEY::BTN_STYLUS2]),
        ]
        .iter()
        .cloned()
        .collect();

        DeviceDispatcher {
            tablet_last_raw_pressed_buttons: 0xFFFF,
            pen_last_raw_pressed_button: 0,
            tablet_button_id_to_key_code_map: default_tablet_button_id_to_key_code_map.clone(),
            pen_button_id_to_key_code_map: default_pen_button_id_to_key_code_map.clone(),
            virtual_pen: Self::virtual_pen_builder(
                &default_pen_button_id_to_key_code_map
                    .values()
                    .flatten()
                    .cloned()
                    .collect::<Vec<EV_KEY>>(),
            ),
            virtual_keyboard: Self::virtual_keyboard_builder(
                &default_tablet_button_id_to_key_code_map
                    .values()
                    .flatten()
                    .cloned()
                    .collect::<Vec<EV_KEY>>(),
            ),
            was_touching: false,
        }
    }

    pub fn syn(&self) -> Result<(), Error> {
        self.virtual_keyboard.syn()?;
        self.virtual_pen.syn()?;
        Ok(())
    }

    pub fn dispatch(&mut self, raw_data: &RawDataReader) {
        self.emit_pen_events(raw_data);
        self.emit_tablet_events(raw_data);
    }

    fn emit_tablet_events(&mut self, raw_data: &RawDataReader) {
        let raw_button_as_binary_flags = raw_data.tablet_buttons_as_binary_flags();
        self.binary_flags_to_tablet_key_events(raw_button_as_binary_flags);
        self.tablet_last_raw_pressed_buttons = raw_button_as_binary_flags;
    }

    fn virtual_keyboard_builder(tablet_emitted_keys: &[EV_KEY]) -> VirtualDevice {
        let mut vd = VirtualDeviceBuilder::new("virtual_keyboard")
            .expect("Error initializig virtual keyboard.");
        vd.enable_keys(tablet_emitted_keys)
            .expect("Error enablig keys for virtual keyboard.")
            .build()
            .expect("Error creating virtual keyboard.")
    }

    fn binary_flags_to_tablet_key_events(&mut self, raw_button_as_flags: u16) {
        (0..14)
            .filter(|i| ![10, 11].contains(i))
            .for_each(|i| self.emit_tablet_key_event(i, raw_button_as_flags));
    }

    pub fn emit_tablet_key_event(&mut self, i: u8, raw_button_as_flags: u16) {
        let id_as_binary_mask = 1 << i;
        let is_pressed = (raw_button_as_flags & id_as_binary_mask) == 0;
        let was_pressed = (self.tablet_last_raw_pressed_buttons & id_as_binary_mask) == 0;

        if let Some(state) = match (was_pressed, is_pressed) {
            (false, true) => Some(Self::PRESSED),
            (true, false) => Some(Self::RELEASED),
            _ => None,
        } {
            if let Some(keys) = self.tablet_button_id_to_key_code_map.get(&i) {
                for &key in keys {
                    if self
                        .virtual_keyboard
                        .emit(EventCode::EV_KEY(key), state)
                        .is_err()
                    {
                        println!("Error emitting vitual keyboard key.");
                    }
                }
                if self.virtual_keyboard.syn().is_err() {
                    println!("Error emitting SYN.");
                };
                self.tablet_last_raw_pressed_buttons = raw_button_as_flags;
            }
        };
    }

    fn virtual_pen_builder(pen_emitted_keys: &[EV_KEY]) -> VirtualDevice {
        let abs_info_x = AbsInfo {
            value: 0,
            minimum: 0,
            maximum: 4096,
            fuzz: 0,
            flat: 0,
            resolution: 1,
        };

        let abs_info_y = AbsInfo {
            value: 0,
            minimum: 0,
            maximum: 4096,
            fuzz: 0,
            flat: 0,
            resolution: 1,
        };

        let abs_info_pressure = AbsInfo {
            value: 0,
            minimum: 0,
            maximum: 1024,
            fuzz: 0,
            flat: 0,
            resolution: 1,
        };

        let mut vd = VirtualDeviceBuilder::new("virtual_pen").expect("Error creating virtual pen.");

        vd.enable_keys(pen_emitted_keys)
            .expect("Error enabling keys for virtual pen.")
            .enable_keys(&[EV_KEY::BTN_TOOL_PEN])
            .expect("Error enabling keys for virtual pen.")
            .enable_abs(EV_ABS::ABS_X, abs_info_x)
            .expect("Error enabling X axis for pen.")
            .enable_abs(EV_ABS::ABS_Y, abs_info_y)
            .expect("Error enabling Y axis for pen.")
            .enable_abs(EV_ABS::ABS_PRESSURE, abs_info_pressure)
            .expect("Error enabling pressure for pen.")
            .build()
            .expect("Error building virtual pen.")
    }

    fn emit_pen_events(&mut self, raw_data: &RawDataReader) {
        let raw_pen_buttons = raw_data.pen_buttons();
        self.raw_pen_buttons_to_pen_key_events(raw_pen_buttons);
        self.pen_last_raw_pressed_button = raw_pen_buttons;
        let normalized_pressure = Self::normalize_pressure(raw_data.pressure());
        self.raw_pen_abs_to_pen_abs_events(
            raw_data.x_axis(),
            raw_data.y_axis(),
            normalized_pressure,
        );

        self.pen_emit_touch(raw_data);
    }

    fn normalize_pressure(raw_pressure: i32) -> i32 {
        match 1740 - raw_pressure {
            x if x <= 0 => 0,
            x => x,
        }
    }

    fn raw_pen_abs_to_pen_abs_events(&mut self, x_axis: i32, y_axis: i32, pressure: i32) {
        if self
            .virtual_pen
            .emit(EventCode::EV_ABS(EV_ABS::ABS_X), x_axis)
            .is_err()
        {
            println!("Error emmitting X value.");
        }
        if self
            .virtual_pen
            .emit(EventCode::EV_ABS(EV_ABS::ABS_Y), y_axis)
            .is_err()
        {
            println!("Error emmitting Y value.");
        }
        if self
            .virtual_pen
            .emit(EventCode::EV_ABS(EV_ABS::ABS_PRESSURE), pressure)
            .is_err()
        {
            println!("Error emmitting Pressure value.");
        }
    }

    fn pen_emit_touch(&mut self, raw_data: &RawDataReader) {
        let is_touching = Self::normalize_pressure(raw_data.pressure()) > 0;
        if let Some(state) = match (self.was_touching, is_touching) {
            (false, true) => Some(Self::PRESSED),
            (true, false) => Some(Self::RELEASED),
            _ => None,
        } {
            if self
                .virtual_pen
                .emit(EventCode::EV_KEY(EV_KEY::BTN_TOUCH), state)
                .is_err()
            {
                println!("Error emmitting Touch state.");
            }
        }
        self.was_touching = is_touching;
    }

    fn raw_pen_buttons_to_pen_key_events(&mut self, pen_button: u8) {
        if let Some((state, id)) = match (self.pen_last_raw_pressed_button, pen_button) {
            (2, x) if x == 6 || x == 4 => Some((Self::PRESSED, x)),
            (x, 2) if x == 6 || x == 4 => Some((Self::RELEASED, x)),
            (x, y) if x != 2 && x == y => Some((Self::HOLD, x)),
            _ => None,
        } {
            let keys = self
                .pen_button_id_to_key_code_map
                .get(&id)
                .expect("Error mapping pen keys.");
            for key in keys {
                self.virtual_pen
                    .emit(EventCode::EV_KEY(*key), state)
                    .expect("Erro emitting key for pen.");
                println!("{}", state);
            }
        }
    }
}
