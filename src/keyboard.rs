#[rustfmt::skip]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Key {
    K0, K1, K2, K3,
    K4, K5, K6, K7,
    K8, K9, KA, KB,
    KC, KD, KE, KF,
}

impl Key {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x0 => Some(Key::K0),
            0x1 => Some(Key::K1),
            0x2 => Some(Key::K2),
            0x3 => Some(Key::K3),
            0x4 => Some(Key::K4),
            0x5 => Some(Key::K5),
            0x6 => Some(Key::K6),
            0x7 => Some(Key::K7),
            0x8 => Some(Key::K8),
            0x9 => Some(Key::K9),
            0xA => Some(Key::KA),
            0xB => Some(Key::KB),
            0xC => Some(Key::KC),
            0xD => Some(Key::KD),
            0xE => Some(Key::KE),
            0xF => Some(Key::KF),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct Keyboard {
    key: Option<Key>,
}

impl Keyboard {
    pub fn set_key(&mut self, key: Key) {
        self.key = Some(key);
    }

    pub fn unset_key(&mut self) {
        self.key.take();
    }

    pub fn is_pressed(&self, key: u8) -> bool {
        self.key == Key::from_u8(key)
    }

    pub fn get_pressed(&self) -> Option<u8> {
        self.key.map(|key| key as u8)
    }
}
