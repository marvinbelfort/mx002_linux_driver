use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AbsInfo, AbsoluteAxisType, AttributeSet, EventType, InputEvent, InputEventKind, Key,
    UinputAbsSetup,
};
use std::{collections::HashMap, error::Error, fmt::Error as Fmterror, u16};

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
        ((high | 0xcc) as u16) << 8 | low as u16
    }

    fn x_axis(&self) -> u16 {
        self.u16_from_2_u8(self.data[Self::X_AXIS_HIGH], self.data[Self::X_AXIS_LOW])
    }

    fn y_axis(&self) -> u16 {
        self.u16_from_2_u8(self.data[Self::Y_AXIS_HIGH], self.data[Self::Y_AXIS_LOW])
    }

    fn pressure(&self) -> u16 {
        self.u16_from_2_u8(
            self.data[Self::PRESSURE_HIGH],
            self.data[Self::PRESSURE_LOW],
        )
    }

    fn tablet_flags(&self) -> u16 {
        self.u16_from_2_u8(
            self.data[Self::TABLET_BUTTONS_HIGH],
            self.data[Self::TABLET_BUTTONS_LOW],
        )
    }

    fn pen_buttons(&self) -> u8 {
        self.data[Self::PEN_BUTTONS]
    }
}

pub struct DeviceDispatcher {
    tablet_last_buttons: u16,
    pen_last_buttons: u8,
    tablet_id_to_key_map: HashMap<u8, Key>,
    pen_id_to_key_map: HashMap<u8, Key>,
    virtual_device: VirtualDevice,
}

impl DeviceDispatcher {
    pub fn new() -> Self {
        let tablet_buttons_ids: Vec<u8> = (0..14).filter(|i| ![10, 11].contains(i)).collect();
        let default_tablet_keys: Vec<Key> = vec![
            Key::KEY_TAB,
            Key::KEY_SPACE,
            Key::KEY_LEFTALT,
            Key::KEY_LEFTCTRL,
            Key::KEY_SCROLLDOWN,
            Key::KEY_SCROLLUP,
            Key::KEY_LEFTBRACE,
            Key::KEY_KPMINUS,
            Key::KEY_KPPLUS,
            Key::KEY_E,
            Key::KEY_B,
            Key::KEY_RIGHTBRACE,
        ];

        let default_pen_keys: Vec<Key> = vec![Key::BTN_STYLUS, Key::BTN_STYLUS2];

        let pen_buttons_ids: Vec<u8> = vec![4, 6];

        DeviceDispatcher {
            tablet_last_buttons: 0,
            pen_last_buttons: 0,
            tablet_id_to_key_map: tablet_buttons_ids
                .into_iter()
                .zip(default_tablet_keys.clone().into_iter())
                .collect(),
            pen_id_to_key_map: pen_buttons_ids
                .into_iter()
                .zip(default_pen_keys.clone().into_iter())
                .collect(),
            virtual_device: Self::virtual_device_builder(&default_pen_keys, &default_tablet_keys)
                .expect("Error creating Virtual Device"),
        }
    }

    pub fn dispatch(&mut self, raw_data: &RawDataReader) -> () {
        println!(
            "{}",
            self.virtual_device
                .get_syspath()
                .expect("No Path Buff")
                .as_path()
                .to_str()
                .expect("Caralho")
        );
        self.emit_tablet_events(raw_data);
        self.emit_pen_events(raw_data);
    }

    fn emit_tablet_events(&mut self, raw_data: &RawDataReader) -> () {
        let raw_button_as_flags = raw_data.tablet_flags();
        self.binary_flags_to_tablet_key_events(raw_button_as_flags);
        self.tablet_last_buttons = raw_button_as_flags;
    }

    fn emit_pen_events(&mut self, raw_data: &RawDataReader) -> () {
        let raw_pen_buttons = raw_data.pen_buttons();
        self.raw_pen_buttons_to_pen_key_events(raw_pen_buttons);
        self.pen_last_buttons = raw_pen_buttons;
        self.raw_pen_abs_to_pen_abs_events(
            raw_data.x_axis(),
            raw_data.y_axis(),
            raw_data.pressure(),
        );
    }

    fn virtual_device_builder(
        pen_keys: &[Key],
        tablet_keys: &[Key],
    ) -> Result<VirtualDevice, std::io::Error> {
        let abs_x_setup =
            UinputAbsSetup::new(AbsoluteAxisType::ABS_X, AbsInfo::new(0, 0, 4096, 0, 0, 1));
        let abs_y_setup =
            UinputAbsSetup::new(AbsoluteAxisType::ABS_Y, AbsInfo::new(0, 0, 4096, 0, 0, 1));
        let abs_pressure_setup = UinputAbsSetup::new(
            AbsoluteAxisType::ABS_PRESSURE,
            AbsInfo::new(0, 0, 1024, 0, 0, 1),
        );

        let mut key_set = AttributeSet::<Key>::new();
        for pen_key in pen_keys {
            key_set.insert(*pen_key);
        }
        for tablet_keys in tablet_keys {
            key_set.insert(*tablet_keys);
        }

        VirtualDeviceBuilder::new()?
            .name("virtual_tablet")
            .with_absolute_axis(&abs_x_setup)?
            .with_absolute_axis(&abs_y_setup)?
            .with_absolute_axis(&abs_pressure_setup)?
            .with_keys(&key_set)?
            .build()
    }

    pub fn emit_tablet_event(&mut self, i: u8, raw_button_as_flags: u16) -> () {
        let mask = 1 << i;
        let is_pressed = (raw_button_as_flags & mask) == 0;
        let was_pressed = (self.tablet_last_buttons & mask) == 0;

        match (was_pressed, is_pressed) {
            (false, true) => Some(0),
            (true, false) => Some(1),
            (true, true) => Some(2),
            (false, false) => None,
        }
        .map(|state| {
            let emit_key = self
                .tablet_id_to_key_map
                .get(&i)
                .expect("Error mapping key")
                .code();
            self.virtual_device
                .emit(&[InputEvent::new(EventType::KEY, emit_key, state)])
                .expect("Error Emmiting");
        });
    }

    fn binary_flags_to_tablet_key_events(&mut self, raw_button_as_flags: u16) -> () {
        (0..14)
            .filter(|i| ![10, 11].contains(i))
            .map(|i| self.emit_tablet_event(i, raw_button_as_flags));
    }

    fn raw_pen_buttons_to_pen_key_events(&mut self, pen_buttons: u8) -> () {
        match (self.pen_last_buttons, pen_buttons) {
            (2, x) if x == 6 || x == 4 => Some((0, x)),
            (x, 2) if x == 6 || x == 4 => Some((1, x)),
            (x, y) if x != 2 && x == y => Some((2, x)),
            _ => None,
        }
        .map(|(state, id)| {
            let emit_key = self
                .pen_id_to_key_map
                .get(&id)
                .expect("Mapping Pen Id to key map")
                .code();
            self.virtual_device
                .emit(&[InputEvent::new(EventType::KEY, emit_key, state)])
        });
    }

    fn raw_pen_abs_to_pen_abs_events(&mut self, x_axis: u16, y_axis: u16, pressure: u16) -> () {
        self.virtual_device.emit(&[
            InputEvent::new(
                EventType::ABSOLUTE,
                AbsoluteAxisType::ABS_X.0,
                x_axis as i32,
            ),
            InputEvent::new(
                EventType::ABSOLUTE,
                AbsoluteAxisType::ABS_Y.0,
                y_axis as i32,
            ),
            InputEvent::new(
                EventType::ABSOLUTE,
                AbsoluteAxisType::ABS_PRESSURE.0,
                pressure as i32,
            ),
        ]);
    }
}
