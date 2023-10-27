use crate::{chip::Chip, keyboard::Key};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct WasmChip {
    chip: Chip,
}

#[wasm_bindgen]
impl WasmChip {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            chip: Chip::default(),
        }
    }

    pub fn reset(&mut self) {
        self.chip.reset();
    }

    pub fn load(&mut self, program: &[u8]) {
        self.chip.load(program);
    }

    pub fn fetch(&mut self) -> u16 {
        self.chip.fetch()
    }

    pub fn execute(&mut self, opcode: u16) {
        self.chip.execute(opcode);
    }

    pub fn tick(&mut self) {
        self.chip.tick();
    }

    pub fn update_timers(&mut self) {
        self.chip.update_timers();
    }

    pub fn has_display_changes(&self) -> bool {
        self.chip.display.get_changes().is_some()
    }

    pub fn ptr_display_buffer(&self) -> *const u8 {
        self.chip.display.get_buffer().as_ptr()
    }

    pub fn set_key(&mut self, key: u8) {
        if let Some(key) = Key::from_u8(key) {
            self.chip.keyboard.set_key(key);
        }
    }

    pub fn unset_key(&mut self) {
        self.chip.keyboard.unset_key();
    }

    pub fn get_key(&self) -> Option<u8> {
        self.chip.keyboard.get_pressed()
    }
}
