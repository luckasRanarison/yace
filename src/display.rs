pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

#[derive(Debug)]
pub struct DisplayChange {
    pub x: usize,
    pub y: usize,
    pub n: usize,
}

#[derive(Debug)]
pub struct Display {
    buffer: [u8; WIDTH * HEIGHT],
    changes: Option<DisplayChange>,
}

impl Default for Display {
    fn default() -> Self {
        Self {
            buffer: [0; WIDTH * HEIGHT],
            changes: None,
        }
    }
}

impl Display {
    pub fn load_sprite(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let mut collision = false;
        let n = sprite.len();

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

        self.changes = Some(DisplayChange { x, y, n });

        collision
    }

    pub fn clear(&mut self) {
        self.buffer = [0; WIDTH * HEIGHT];
    }

    pub fn get_buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn get_changes(&self) -> Option<&DisplayChange> {
        self.changes.as_ref()
    }

    pub fn drop_changes(&mut self) {
        self.changes.take();
    }
}
