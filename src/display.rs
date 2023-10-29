pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

#[derive(Debug)]
pub struct Display {
    buffer: [u8; WIDTH * HEIGHT],
    updated: bool,
}

impl Default for Display {
    fn default() -> Self {
        Self {
            buffer: [0; WIDTH * HEIGHT],
            updated: false,
        }
    }
}

impl Display {
    pub fn load_sprite(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let mut collision = false;

        for (i, row) in sprite.iter().enumerate() {
            for offset in 0..8 {
                let new = row >> (7 - offset) & 1;
                let py = (y + i) % HEIGHT;
                let px = (x + offset) % WIDTH;
                let index = (py * WIDTH) + px;
                let old = self.buffer[index];

                self.buffer[index] = new ^ old;

                if !collision && new == 1 && new ^ old == 0 {
                    collision = true;
                }
            }
        }

        self.updated = true;

        collision
    }

    pub fn clear(&mut self) {
        self.buffer = [0; WIDTH * HEIGHT];
    }

    pub fn get_buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn has_changed(&self) -> bool {
        self.updated
    }

    pub fn clear_status(&mut self) {
        self.updated = false
    }
}
