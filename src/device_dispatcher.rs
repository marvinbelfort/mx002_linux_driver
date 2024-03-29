use evdev::{uinput::VirtualDeviceBuilder, Key};
use std::collections::HashMap;

enum PressState {
    Pressed = 0,
    Release = 1,
    Hold = 2,
}

pub struct PhysicalButton {
    id: u8,
    state: PressState,
}

pub struct DeviceDispatcher {
    tablet_last_buttons: u16,
    pen_last_buttons: u8,
    tablet_id_to_key_map: HashMap<u8, Key>,
    pen_id_to_key_map: HashMap<u8, Key>,
}

impl DeviceDispatcher {
    const X_AXIS_HIGH: usize = 1;
    const X_AXIS_LOW: usize = 2;
    const Y_AXIS_HIGH: usize = 3;
    const Y_AXIS_LOW: usize = 4;
    const Y_PRESSURE_HIGH: usize = 5;
    const Y_PRESSURE_LOW: usize = 6;
    const PEN_BUTTONS: usize = 9;
    const TABLET_BUTTONS_HIGH: usize = 12;
    const TABLET_BUTTONS_LOW: usize = 11;

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

        let default_pen_keys: Vec<Key> = vec![
            Key::BTN_STYLUS,
            Key::BTN_STYLUS2,
        ];

        let pen_buttons_ids: Vec<u8> = vec![4, 6];

        DeviceDispatcher {
            tablet_last_buttons: 0,
            pen_last_buttons: 0,
            tablet_id_to_key_map: tablet_buttons_ids
                .into_iter()
                .zip(default_tablet_keys.into_iter())
                .collect(),
            pen_id_to_key_map: pen_buttons_ids.into_iter().zip(default_pen_keys.into_iter()).collect(),
        }
    }

    pub fn dispatch(&mut self, buffer: &[u8]) -> () {
        let binary_button_flags = Self::button_flags(
            buffer[Self::TABLET_BUTTONS_HIGH],
            buffer[Self::TABLET_BUTTONS_LOW],
        );

        let tablet_buttons_with_events = self.binary_flags_to_tablet_buttons(binary_button_flags);
        self.tablet_last_buttons = binary_button_flags;

        let pen_button_data = buffer[Self::PEN_BUTTONS];
        let pen_buttons_with_events = self.pen_button_data_to_pen_buttons(pen_button_data);
        self.pen_last_buttons = pen_button_data;

        // Mapeia botões pressionados para KEY_Events
        // Calcula boundaries e Emite X, Y
        // Calcula e emite Pen Pressure
    }

    fn button_flags(high: u8, low: u8) -> u16 {
        ((high | 0xcc) as u16) << 8 | low as u16
    }

    fn binary_flags_to_tablet_buttons(&self, binary_button_flags: u16) -> Vec<PhysicalButton> {
        (0..14)
            .filter(|i| ![10, 11].contains(i))
            .filter_map(|i| self.tablet_button_from_bits(i, binary_button_flags))
            .collect()
    }

    pub fn tablet_button_from_bits(
        &self,
        i: u8,
        binary_button_flags: u16,
    ) -> Option<PhysicalButton> {
        let mask = 1 << i;
        let is_pressed = (binary_button_flags & mask) == 0;
        let was_pressed = (self.tablet_last_buttons & mask) == 0;

        match (was_pressed, is_pressed) {
            (true, true) => Some(PressState::Hold),
            (false, true) => Some(PressState::Pressed),
            (true, false) => Some(PressState::Release),
            (false, false) => None,
        }
        .map(|state| PhysicalButton { id: i, state })
    }

    fn pen_button_data_to_pen_buttons(&self, pen_buttons: u8) -> Option<PhysicalButton> {
        match (self.pen_last_buttons, pen_buttons) {
            (2, x) if x == 6 || x == 4 => Some(PressState::Pressed),
            (x, 2) if x == 6 || x == 4 => Some(PressState::Release),
            (x, y) if x == y => Some(PressState::Hold),
            _ => None,
        }
        .map(|state| PhysicalButton {
            id: pen_buttons,
            state,
        })
    }
}
