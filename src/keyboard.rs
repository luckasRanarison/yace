#[derive(Debug, Default)]
pub struct Keyboard {
    pad: [bool; 16],
}

impl Keyboard {
    pub fn set_key(&mut self, key: u8) {
        self.pad[key as usize] = true;
    }

    pub fn unset_key(&mut self, key: u8) {
        self.pad[key as usize] = false;
    }

    pub fn is_pressed(&self, key: u8) -> bool {
        self.pad[key as usize]
    }

    pub fn get_pressed(&self) -> Option<usize> {
        self.pad.iter().position(|&key| key)
    }
}
