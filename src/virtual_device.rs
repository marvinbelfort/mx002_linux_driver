use std::io::Error;
use std::{collections::HashMap, u16};

use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AbsInfo, AbsoluteAxisType, AttributeSet, EventType, InputEvent, Key, Synchronization,
    UinputAbsSetup,
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

/* pub struct VirtualDeviceWrapper {
    uinput_device: UInputDevice,
}

impl VirtualDeviceWrapper {
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
} */

/* pub struct VirtualDeviceBuilder {
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

    pub fn enable_pointer(&mut self) -> Result<&mut Self, Error> {
        if self
            .uninit_device
            .enable(InputProp::INPUT_PROP_POINTER)
            .is_err()
        {
            println!("Error enabling device as Pointer.");
        }
        Ok(self)
    }

    pub fn enable_keys(&mut self, keys: &[Key]) -> Result<&mut Self, Error> {
        for &key in keys {
            if self.uninit_device.enable(EventCode::Key(key)).is_err() {
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

    pub fn build(&mut self) -> Result<VirtualDeviceWrapper, Error> {
        self.uninit_device
            .enable(EventCode::EV_SYN(EV_SYN::SYN_REPORT))?;
        let uinput_device = UInputDevice::create_from_device(&self.uninit_device)?;
        Ok(VirtualDeviceWrapper { uinput_device })
    }
} */

pub struct DeviceDispatcher {
    tablet_last_raw_pressed_buttons: u16,
    pen_last_raw_pressed_button: u8,
    tablet_button_id_to_key_code_map: HashMap<u8, Vec<Key>>,
    pen_button_id_to_key_code_map: HashMap<u8, Vec<Key>>,
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
        let default_tablet_button_id_to_key_code_map: HashMap<u8, Vec<Key>> = [
            (0, vec![Key::KEY_TAB]),
            (1, vec![Key::KEY_SPACE]),
            (2, vec![Key::KEY_LEFTALT]),
            (3, vec![Key::KEY_LEFTCTRL]),
            (4, vec![Key::KEY_SCROLLDOWN]),
            (5, vec![Key::KEY_SCROLLUP]),
            (6, vec![Key::KEY_LEFTBRACE]),
            (7, vec![Key::KEY_LEFTCTRL, Key::KEY_KPMINUS]),
            (8, vec![Key::KEY_KPPLUS]),
            (9, vec![Key::KEY_E]),
            //10 This code is not emitted by physical device
            //11 This code is not emitted by physical device
            (12, vec![Key::KEY_B]),
            (13, vec![Key::KEY_RIGHTBRACE]),
        ]
        .iter()
        .cloned()
        .collect();

        let default_pen_button_id_to_key_code_map: HashMap<u8, Vec<Key>> =
            [(4, vec![Key::BTN_STYLUS]), (6, vec![Key::BTN_STYLUS2])]
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
                    .collect::<Vec<Key>>(),
            )
            .expect("Error building virtual pen"),
            virtual_keyboard: Self::virtual_keyboard_builder(
                &default_tablet_button_id_to_key_code_map
                    .values()
                    .flatten()
                    .cloned()
                    .collect::<Vec<Key>>(),
            )
            .expect("Error building virtual keyborad"),
            was_touching: false,
        }
    }

    pub fn syn(&mut self) -> Result<(), Error> {
        self.virtual_keyboard.emit(&[InputEvent::new(
            EventType::SYNCHRONIZATION,
            Synchronization::SYN_REPORT.0,
            0,
        )])?;
        self.virtual_pen.emit(&[InputEvent::new(
            EventType::SYNCHRONIZATION,
            Synchronization::SYN_REPORT.0,
            0,
        )])?;
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

    fn virtual_keyboard_builder(tablet_emitted_keys: &[Key]) -> Result<VirtualDevice, Error> {
        let mut key_set = AttributeSet::<Key>::new();
        for key in tablet_emitted_keys {
            key_set.insert(*key);
        }

        VirtualDeviceBuilder::new()?
            .name("virtual_tablet")
            .with_keys(&key_set)?
            .build()
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
            (true, true) => Some(Self::HOLD),
            _ => None,
        } {
            if let Some(keys) = self.tablet_button_id_to_key_code_map.get(&i) {
                for &key in keys {
                    self.virtual_keyboard
                        .emit(&[InputEvent::new(EventType::KEY, key.code(), state)])
                        .expect("Error emitting vitual keyboard key.");
                }

                self.virtual_keyboard
                    .emit(&[InputEvent::new(
                        EventType::SYNCHRONIZATION,
                        Synchronization::SYN_REPORT.0,
                        0,
                    )])
                    .expect("Error emitting SYN.");
            }
        };
    }

    fn virtual_pen_builder(pen_emitted_keys: &[Key]) -> Result<VirtualDevice, Error> {
        let abs_x_setup =
            UinputAbsSetup::new(AbsoluteAxisType::ABS_X, AbsInfo::new(0, 0, 4096, 0, 0, 1));
        let abs_y_setup =
            UinputAbsSetup::new(AbsoluteAxisType::ABS_Y, AbsInfo::new(0, 0, 4096, 0, 0, 1));
        let abs_pressure_setup = UinputAbsSetup::new(
            AbsoluteAxisType::ABS_PRESSURE,
            AbsInfo::new(0, 0, 600, 0, 0, 1),
        );

        let mut key_set = AttributeSet::<Key>::new();
        for key in pen_emitted_keys {
            key_set.insert(*key);
        }

        for key in &[Key::BTN_TOOL_PEN, Key::BTN_LEFT, Key::BTN_RIGHT] {
            key_set.insert(*key);
        }

        VirtualDeviceBuilder::new()?
            .name("virtual_tablet")
            .with_absolute_axis(&abs_x_setup)?
            .with_absolute_axis(&abs_y_setup)?
            .with_absolute_axis(&abs_pressure_setup)?
            .with_keys(&key_set)?
            .build()
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
        self.virtual_pen.emit(&[InputEvent::new(
            EventType::ABSOLUTE,
            AbsoluteAxisType::ABS_X.0,
            x_axis as i32,
        )]);
        self.virtual_pen.emit(&[InputEvent::new(
            EventType::ABSOLUTE,
            AbsoluteAxisType::ABS_Y.0,
            y_axis as i32,
        )]);
        self.virtual_pen.emit(&[InputEvent::new(
            EventType::ABSOLUTE,
            AbsoluteAxisType::ABS_PRESSURE.0,
            pressure as i32,
        )]);
    }

    fn pen_emit_touch(&mut self, raw_data: &RawDataReader) {
        let is_touching = Self::normalize_pressure(raw_data.pressure()) > 0;
        if let Some(state) = match (self.was_touching, is_touching) {
            (false, true) => Some(Self::PRESSED),
            (true, false) => Some(Self::RELEASED),
            _ => None,
        } {
            self.virtual_pen.emit(&[InputEvent::new(
                EventType::KEY,
                Key::BTN_TOUCH.code(),
                state,
            )]);
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
                    .emit(&[InputEvent::new(EventType::KEY, key.code(), state)])
                    .expect("Error emitting pen keys.")
            }
        }
    }
}
