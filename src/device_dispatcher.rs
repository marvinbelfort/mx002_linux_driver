
use evdev::{uinput::VirtualDeviceBuilder, Key};

enum PressState {
    Pressed = 0,
    Release = 1,
    Hold = 2,
    None = 3,
}

pub struct TabletButton {
    id: u8,
    state: PressState,
}

pub struct DeviceDispatcher {
    tablet_last_buttons: u16,
    pen_last_buttons: u8,
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
        let default_tablet_keys: Vec<Key> = vec![Key::KEY_TAB, Key::KEY_SPACE, Key::KEY_LEFTALT, Key::KEY_LEFTCTRL,];
        DeviceDispatcher {
            tablet_last_buttons: 0,
            pen_last_buttons: 0,
        }
    }

    pub fn dispatch(&mut self, buffer: &[u8]) -> () {
        let binary_button_flags = Self::button_flags(
            buffer[Self::TABLET_BUTTONS_HIGH],
            buffer[Self::TABLET_BUTTONS_LOW],
        );

        let buttons = self.binary_to_buttons(binary_button_flags);
        self.tablet_last_buttons = binary_button_flags;

        // Mapeia botões pressionados para KEY_Events
        // Calcula boundaries e Emite X, Y
        // Calcula e emite Pen Pressure
    }

    fn button_flags(high: u8, low: u8) -> u16 {
        ((high | 0xcc) as u16) << 8 | low as u16
    }

    fn binary_to_buttons(&self, binary_button_flags: u16) -> Vec<TabletButton> {
        (0..14)
            .filter(|i| ![10, 11].contains(i))
            .map(|i| self.buttons_from_bits(i, binary_button_flags))
            .collect()
    }

    pub fn buttons_from_bits(&self, i: u8, binary_button_flags: u16) -> TabletButton {
        let mask = 1 << i;
        let is_pressed = (binary_button_flags & mask) == 0;
        let was_pressed = (self.tablet_last_buttons & mask) == 0;

        TabletButton {
            id: i,
            state: {
                match (was_pressed, is_pressed) {
                    (true, true) => PressState::Hold,
                    (false, true) => PressState::Pressed,
                    (false, false) => PressState::None,
                    (true, false) => PressState::Release,
                }
            },
        }
    }
}
