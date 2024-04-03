use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AbsInfo, AbsoluteAxisType, AttributeSet, EventType, InputEvent, Key, Synchronization,
    UinputAbsSetup,
};
use std::{collections::HashMap, u16};

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

pub struct DeviceDispatcher {
    tablet_last_raw_pressed_buttons: u16,
    pen_last_raw_pressed_buttons: u8,
    map_tablet_button_id_to_emitted_key: HashMap<u8, Key>,
    map_pen_button_id_to_emitted_key: HashMap<u8, Key>,
    virtual_device: VirtualDevice,
    was_touching: bool,
}

impl DeviceDispatcher {
    pub fn new() -> Self {
        let tablet_buttons_ids: Vec<u8> = (0..14).filter(|i| ![10, 11].contains(i)).collect();
        let default_tablet_emitted_keys: Vec<Key> = vec![
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

        let pen_buttons_ids: Vec<u8> = vec![4, 6];
        let default_pen_emitted_keys: Vec<Key> = vec![Key::BTN_STYLUS, Key::BTN_STYLUS2];

        DeviceDispatcher {
            tablet_last_raw_pressed_buttons: 0,
            pen_last_raw_pressed_buttons: 0,
            map_tablet_button_id_to_emitted_key: tablet_buttons_ids
                .into_iter()
                .zip(default_tablet_emitted_keys.clone())
                .collect(),
            map_pen_button_id_to_emitted_key: pen_buttons_ids
                .into_iter()
                .zip(default_pen_emitted_keys.clone())
                .collect(),
            virtual_device: Self::virtual_device_builder(
                &default_pen_emitted_keys,
                &default_tablet_emitted_keys,
            )
            .expect("Error creating Virtual Device"),
            was_touching: false,
        }
    }

    pub fn dispatch(&mut self, raw_data: &RawDataReader) {
        self.emit_tablet_events(raw_data);
        self.emit_pen_events(raw_data);
    }

    fn emit_tablet_events(&mut self, raw_data: &RawDataReader) {
        let raw_button_as_binary_flags = raw_data.tablet_buttons_as_binary_flags();
        self.binary_flags_to_tablet_key_events(raw_button_as_binary_flags);
        self.tablet_last_raw_pressed_buttons = raw_button_as_binary_flags;
    }

    fn emit_pen_events(&mut self, raw_data: &RawDataReader) {
        let raw_pen_buttons = raw_data.pen_buttons();
        self.raw_pen_buttons_to_pen_key_events(raw_pen_buttons);
        self.pen_last_raw_pressed_buttons = raw_pen_buttons;
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

    fn virtual_device_builder(
        pen_emitted_keys: &[Key],
        tablet_emitted_keys: &[Key],
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
        for key in pen_emitted_keys {
            key_set.insert(*key);
        }
        for key in tablet_emitted_keys {
            key_set.insert(*key);
        }

        key_set.insert(Key::BTN_TOOL_PEN);

        VirtualDeviceBuilder::new()?
            .name("virtual_tablet_and_stylus")
            .with_absolute_axis(&abs_x_setup)?
            .with_absolute_axis(&abs_y_setup)?
            .with_absolute_axis(&abs_pressure_setup)?
            .with_keys(&key_set)?
            .build()
    }

    pub fn emit_tablet_key_event(&mut self, i: u8, raw_button_as_flags: u16) {
        let id_as_binary_mask = 1 << i;
        let is_pressed = (raw_button_as_flags & id_as_binary_mask) == 0;
        let was_pressed = (self.tablet_last_raw_pressed_buttons & id_as_binary_mask) == 0;

        if let Some(state) = match (was_pressed, is_pressed) {
            (false, true) => Some(0), //Pressed
            (true, false) => Some(1), //Released
            (true, true) => Some(2),  //Hold: hardware does not support, will never emit.
            (false, false) => None,
        } {
            let emit_key = self
                .map_tablet_button_id_to_emitted_key
                .get(&i)
                .expect("Error mapping tablet keys")
                .code();
            self.emit(EventType::KEY, emit_key, state);
        };
    }

    fn raw_pen_buttons_to_pen_key_events(&mut self, pen_buttons: u8) {
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
            self.emit_and_log(EventType::KEY, emit_key, state, "pen btns");
        };
    }

    fn emit(&mut self, event_type: EventType, code: u16, state: i32) {
        self.virtual_device
            .emit(&[InputEvent::new(event_type, code, state)])
            .expect("Error emitting");
        // self.virtual_device
        //     .emit(&[InputEvent::new(
        //         EventType::SYNCHRONIZATION,
        //         Synchronization::SYN_REPORT.0,
        //         0,
        //     )])
        //     .expect("Error emitting SYN");
    }

    fn emit_and_log(&mut self, event_type: EventType, code: u16, state: i32, message: &str) {
        self.emit(event_type, code, state);
        println!("{message}: Type: {event_type:?} Code: {code} State: {state}");
    }

    fn binary_flags_to_tablet_key_events(&mut self, raw_button_as_flags: u16) {
        (0..14)
            .filter(|i| ![10, 11].contains(i))
            .for_each(|i| self.emit_tablet_key_event(i, raw_button_as_flags));
    }

    fn raw_pen_abs_to_pen_abs_events(&mut self, x_axis: i32, y_axis: i32, pressure: i32) {
        self.emit(EventType::ABSOLUTE, AbsoluteAxisType::ABS_X.0, x_axis);
        self.emit(EventType::ABSOLUTE, AbsoluteAxisType::ABS_Y.0, y_axis);
        self.emit(
            EventType::ABSOLUTE,
            AbsoluteAxisType::ABS_PRESSURE.0,
            pressure,
        );
    }

    fn pen_emit_touch(&mut self, raw_data: &RawDataReader) {
        let is_touching = Self::normalize_pressure(raw_data.pressure()) > 0;
        if let Some(state) = match (self.was_touching, is_touching) {
            (false, true) => Some(0), //Pressed
            (true, false) => Some(1), //Released
            _ => None,
        } {
            self.emit_and_log(EventType::KEY, Key::BTN_TOUCH.code(), state, "touch")
        }
        self.was_touching = is_touching;
    }
}
