#[rustfmt::skip]

#[derive(Debug, Default)]
pub struct Keyboard {
    key: [bool; 16],
}

impl Keyboard {
    pub fn reset(&mut self) {
        self.key = [false; 16];
    }

    pub fn set_key(&mut self, key: u8) {
        self.key[key as usize] = true;
    }

    pub fn unset_key(&mut self, key: u8) {
        self.key[key as usize] = false;
    }

    pub fn is_pressed(&self, key: u8) -> bool {
        self.key[key as usize]
    }

    pub fn get_pressed(&self) -> Option<u8> {
        self.key.iter().position(|&key| key).map(|pos| pos as u8)
    }
}
