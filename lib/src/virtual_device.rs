#![allow(unused, dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};
use std::{cell::RefCell, collections::HashMap, rc::Rc, u16};

use evdev_rs::enums::{EventCode, EV_ABS, EV_KEY, EV_SYN};
use evdev_rs::{DeviceWrapper, InputEvent, TimeVal, UInputDevice, UninitDevice};

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
    pub fn emit(&self, event_code: EventCode, value: i32) -> Result<(), std::io::Error> {
        self.uinput_device.write_event(&InputEvent {
            time: VirtualDevice::now(),
            event_code,
            value,
        })?;
        Ok(())
    }

    pub fn syn(&self) -> Result<(), std::io::Error> {
        self.emit(EventCode::EV_SYN(EV_SYN::SYN_REPORT), 1)?;
        Ok(())
    }

    fn now() -> TimeVal {
        let current_time = SystemTime::now();
        let duration_since_epoch = current_time
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let tv_sec = duration_since_epoch.as_secs() as i64; // `time_t` geralmente é um i64
        let tv_usec = duration_since_epoch.subsec_micros() as i64; // `suseconds_t` geralmente é um i64 ou i32, dependendo do sistema

        TimeVal::new(tv_sec, tv_usec)
    }
}

pub struct VirtualDeviceBuilder {
    uninit_device: UninitDevice,
}

impl VirtualDeviceBuilder {
    pub fn new(name: &str) -> Self {
        let uninit_device = UninitDevice::new().unwrap();
        uninit_device.set_name(name);
        VirtualDeviceBuilder { uninit_device }
    }

    pub fn enable_keys(& mut self, keys: &[EV_KEY]) -> &mut Self {
        for &key in keys {
            self.uninit_device.enable(EventCode::EV_KEY(key));
        }
        self
    }

    pub fn enable_abs(& mut self, absolutes: &[EV_ABS]) -> & mut Self {
        for &absolute in absolutes {
            self.uninit_device.enable(EventCode::EV_ABS(absolute));
        }
        self
    }

    pub fn build(&mut self) -> VirtualDevice {
        self.uninit_device
            .enable(EventCode::EV_SYN(EV_SYN::SYN_REPORT));
        let uinput_device = UInputDevice::create_from_device(&self.uninit_device).unwrap();
        VirtualDevice { uinput_device }
    }
}

pub struct DeviceDispatcher {
    tablet_last_raw_pressed_buttons: u16,
    pen_last_raw_pressed_buttons: u8,
    map_tablet_button_id_to_emitted_key: HashMap<u8, EV_KEY>,
    map_pen_button_id_to_emitted_key: HashMap<u8, EV_KEY>,
    // virtual_pen: VirtualDevice,
    virtual_keyboard: VirtualDevice,
    was_touching: bool,
}

impl DeviceDispatcher {
    pub fn new() -> Self {
        let tablet_buttons_ids: Vec<u8> = (0..14).filter(|i| ![10, 11].contains(i)).collect();
        let mut default_tablet_emitted_keys: Vec<EV_KEY> = vec![
            EV_KEY::KEY_TAB,
            EV_KEY::KEY_SPACE,
            EV_KEY::KEY_LEFTALT,
            EV_KEY::KEY_LEFTCTRL,
            EV_KEY::KEY_SCROLLDOWN,
            EV_KEY::KEY_SCROLLUP,
            EV_KEY::KEY_LEFTBRACE,
            EV_KEY::KEY_KPMINUS,
            EV_KEY::KEY_KPPLUS,
            EV_KEY::KEY_E,
            EV_KEY::KEY_B,
            EV_KEY::KEY_RIGHTBRACE,
        ];

        let pen_buttons_ids: Vec<u8> = vec![4, 6];
        let default_pen_emitted_keys: Vec<EV_KEY> = vec![EV_KEY::BTN_STYLUS, EV_KEY::BTN_STYLUS2];

        DeviceDispatcher {
            tablet_last_raw_pressed_buttons: 0xFFFF,
            pen_last_raw_pressed_buttons: 0,
            map_tablet_button_id_to_emitted_key: tablet_buttons_ids
                .into_iter()
                .zip(default_tablet_emitted_keys.clone())
                .collect(),
            map_pen_button_id_to_emitted_key: pen_buttons_ids
                .into_iter()
                .zip(default_pen_emitted_keys.clone())
                .collect(),
            // virtual_pen: Self::virtual_pen_builder(&default_pen_emitted_keys)
            //     .expect("Error creating Virtual Pen"),
            virtual_keyboard: Self::virtual_keyboard_builder(&mut default_tablet_emitted_keys),
            was_touching: false,
        }
    }

    pub fn dispatch(&mut self, raw_data: &RawDataReader) {
        // self.emit_pen_events(raw_data);
        self.emit_tablet_events(raw_data);
    }

    fn emit_tablet_events(&mut self, raw_data: &RawDataReader) {
        let raw_button_as_binary_flags = raw_data.tablet_buttons_as_binary_flags();
        self.binary_flags_to_tablet_key_events(raw_button_as_binary_flags);
        self.tablet_last_raw_pressed_buttons = raw_button_as_binary_flags;
    }

    /* fn emit_pen_events(&mut self, raw_data: &RawDataReader) {
        let raw_pen_buttons = raw_data.pen_buttons();
        self.raw_pen_buttons_to_pen_key_events(raw_pen_buttons);
        self.pen_last_raw_pressed_buttons = raw_pen_buttons;
        let normalized_pressure = Self::normalize_pressure(raw_data.pressure());
        self.raw_pen_abs_to_pen_abs_events(
            raw_data.x_axis(),
            raw_data.y_axis(),
            normalized_pressure,
        );

        // self.pen_emit_touch(raw_data);
    } */

    fn normalize_pressure(raw_pressure: i32) -> i32 {
        match 1740 - raw_pressure {
            x if x <= 0 => 0,
            x => x,
        }
    }

    /* fn virtual_pen_builder(
        pen_emitted_keys: &[Key],
    ) -> Result<Rc<RefCell<VirtualDevice>>, std::io::Error> {
        // let abs_x_setup =
        //     UinputAbsSetup::new(AbsoluteAxisType::ABS_X, AbsInfo::new(0, 0, 4096, 0, 0, 1));
        // let abs_y_setup =
        //     UinputAbsSetup::new(AbsoluteAxisType::ABS_Y, AbsInfo::new(0, 0, 4096, 0, 0, 1));
        // let abs_pressure_setup = UinputAbsSetup::new(
        //     AbsoluteAxisType::ABS_PRESSURE,
        //     AbsInfo::new(0, 0, 1024, 0, 0, 1),
        // );

        let mut key_set = AttributeSet::<Key>::new();

        let mut properties = AttributeSet::<PropType>::new();
        properties.insert(PropType::POINTER);

        key_set.insert(Key::BTN_TOOL_PEN);

        for key in pen_emitted_keys {
            key_set.insert(*key);
        }

        let virtual_device = VirtualDeviceBuilder::new()?
            .name("virtual_pen")
            .with_absolute_axis(&abs_x_setup)?
            .with_absolute_axis(&abs_y_setup)?
            .with_absolute_axis(&abs_pressure_setup)?
            .with_keys(&key_set)?
            .with_properties(&properties)?
            .build()?;

        Ok(Rc::new(RefCell::new(virtual_device)))
    } */

    fn virtual_keyboard_builder(tablet_emitted_keys: &mut[EV_KEY]) -> VirtualDevice {
        let name = "virtual_keyboard";
        let mut vd = VirtualDeviceBuilder::new(name);
        vd.enable_keys(& tablet_emitted_keys).build()
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
            (false, true) => Some(0), //Pressed
            (true, false) => Some(1), //Released
            _ => None,
        } {
            if let Some(&key) = self.map_tablet_button_id_to_emitted_key.get(&i) {
                self.virtual_keyboard.emit(EventCode::EV_KEY(key), state);
                self.virtual_keyboard.syn();
                self.tablet_last_raw_pressed_buttons = raw_button_as_flags;
                println!(
                    "{:016b} is:{:05} was:{:05}[{:016b}] id[{i:02}]{:016b} : {state}",
                    raw_button_as_flags,
                    is_pressed,
                    was_pressed,
                    self.tablet_last_raw_pressed_buttons,
                    id_as_binary_mask
                );
            }
        };
    }

    /* fn raw_pen_abs_to_pen_abs_events(&mut self, x_axis: i32, y_axis: i32, pressure: i32) {
        self.emit(
            &Rc::clone(&self.virtual_pen),
            EventType::ABSOLUTE,
            AbsoluteAxisType::ABS_X.0,
            x_axis,
        );
        self.emit(
            &Rc::clone(&self.virtual_pen),
            EventType::ABSOLUTE,
            AbsoluteAxisType::ABS_Y.0,
            y_axis,
        );
        self.emit(
            &Rc::clone(&self.virtual_pen),
            EventType::ABSOLUTE,
            AbsoluteAxisType::ABS_PRESSURE.0,
            pressure,
        );
    } */

    /* fn pen_emit_touch(&mut self, raw_data: &RawDataReader) {
        let is_touching = Self::normalize_pressure(raw_data.pressure()) > 0;
        if let Some(state) = match (self.was_touching, is_touching) {
            (false, true) => Some(0), //Pressed
            (true, false) => Some(1), //Released
            _ => None,
        } {
            self.emit(
                &Rc::clone(&self.virtual_pen),
                EventType::KEY,
                Key::BTN_TOUCH.code(),
                state,
            );
        }
        self.was_touching = is_touching;
    } */

    /* fn raw_pen_buttons_to_pen_key_events(&mut self, pen_buttons: u8) {
        if let Some((state, id)) = match (self.pen_last_raw_pressed_buttons, pen_buttons) {
            (2, x) if x == 6 || x == 4 => Some((0, x)),
            (x, 2) if x == 6 || x == 4 => Some((1, x)),
            (x, y) if x != 2 && x == y => Some((2, x)),
            _ => None,
        } {
            let emit_key = self
                .map_pen_button_id_to_emitted_key
                .get(&id)
                .expect("Error mapping pen keys")
                .code();
            self.emit(
                &Rc::clone(&self.virtual_pen),
                EventType::KEY,
                emit_key,
                state,
            );
        };
    } */
}
