use crate::{display::Display, keyboard::Keyboard, memory::Memory};
use rand::{rngs::ThreadRng, thread_rng, Rng};

const PRG_START: u16 = 0x200;

#[derive(Debug)]
pub struct Chip {
    v: [u8; 16],
    i: u16,
    dt: u16,
    st: u16,
    pc: u16,
    sp: u8,
    stack: [u16; 16],
    memory: Memory,
    rng: ThreadRng,

    pub display: Display,
    pub keyboard: Keyboard,
}

impl Default for Chip {
    fn default() -> Self {
        Self {
            v: [0; 16],
            i: 0,
            dt: 0,
            st: 0,
            pc: PRG_START,
            sp: 0,
            stack: [0; 16],
            memory: Memory::default(),
            keyboard: Keyboard::default(),
            display: Display::default(),
            rng: thread_rng(),
        }
    }
}

impl Chip {
    pub fn new(program: &[u8]) -> Self {
        let mut chip = Chip::default();

        chip.load(program);
        chip
    }

    pub fn reset(&mut self) {
        self.v = [0; 16];
        self.i = 0;
        self.dt = 0;
        self.st = 0;
        self.pc = PRG_START;
        self.sp = 0;
        self.stack = [0; 16];
        self.memory.clear();
        self.keyboard.reset();
        self.display.clear();
    }

    pub fn load(&mut self, program: &[u8]) {
        let start = PRG_START as usize;
        let end = start + program.len();

        self.memory.write_slice(start, end, program);
    }

    pub fn tick(&mut self) {
        let instruction = self.fetch();

        self.display.clear_status();
        self.execute(instruction);
    }

    pub fn update_timers(&mut self) {
        self.dt = self.dt.saturating_sub(1);
        self.st = self.st.saturating_sub(1);
    }

    pub fn fetch(&self) -> u16 {
        let msb = self.memory.read(self.pc) as u16;
        let lsb = self.memory.read(self.pc + 1) as u16;

        (msb << 8) | lsb
    }

    pub fn execute(&mut self, opcode: u16) {
        let nibbles = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
        );

        let nnn = opcode & 0x0FFF;
        let kk = (opcode & 0x00FF) as u8;

        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => self.cls(),
            (0x0, 0x0, 0xE, 0xE) => self.ret(),
            (0x1, _, _, _) => self.jp_addr(nnn),
            (0x2, _, _, _) => self.call_addr(nnn),
            (0x3, x, _, _) => self.se_vx_b(x, kk),
            (0x4, x, _, _) => self.sne_vx_b(x, kk),
            (0x5, x, y, 0x0) => self.se_vx_vy(x, y),
            (0x6, x, _, _) => self.ld_vx_b(x, kk),
            (0x7, x, _, _) => self.add_vx_b(x, kk),
            (0x8, x, y, 0x0) => self.ld_vx_vy(x, y),
            (0x8, x, y, 0x1) => self.or_vx_vy(x, y),
            (0x8, x, y, 0x2) => self.and_vx_vy(x, y),
            (0x8, x, y, 0x3) => self.xor_vx_vy(x, y),
            (0x8, x, y, 0x4) => self.add_vx_vy(x, y),
            (0x8, x, y, 0x5) => self.sub_vx_vy(x, y),
            (0x8, x, _, 0x6) => self.shr_vx(x),
            (0x8, x, y, 0x7) => self.sub_vy_vx(y, x),
            (0x8, x, _, 0xE) => self.shl_vx(x),
            (0x9, x, y, 0x0) => self.sne_vx_vy(x, y),
            (0xA, _, _, _) => self.ld_i_addr(nnn),
            (0xB, _, _, _) => self.jp_v_addr(nnn),
            (0xC, x, _, _) => self.rnd_vx_b(x, kk),
            (0xD, x, y, n) => self.drw_x_y_n(x, y, n),
            (0xE, x, 0x9, 0xE) => self.skp_vx(x),
            (0xE, x, 0xA, 0x1) => self.sknp_vx(x),
            (0xF, x, 0x0, 0x7) => self.ld_vx_dt(x),
            (0xF, x, 0x0, 0xA) => self.ld_vx_k(x),
            (0xF, x, 0x1, 0x5) => self.ld_dt_vx(x),
            (0xF, x, 0x1, 0x8) => self.ld_st_vx(x),
            (0xF, x, 0x1, 0xE) => self.add_i_vx(x),
            (0xF, x, 0x2, 0x9) => self.ld_f_vx(x),
            (0xF, x, 0x3, 0x3) => self.ld_b_vx(x),
            (0xF, x, 0x5, 0x5) => self.ld_i_vx(x),
            (0xF, x, 0x6, 0x5) => self.ld_vx_i(x),
            _ => {
                panic!("Unsupported instruction: {:x}", opcode);
            }
        }
    }

    fn cls(&mut self) {
        self.display.clear();
        self.increment();
    }

    fn ret(&mut self) {
        self.sp -= 1;
        self.jump(self.stack[self.sp as usize]);
    }

    fn jp_addr(&mut self, addr: u16) {
        self.jump(addr);
    }

    fn jp_v_addr(&mut self, addr: u16) {
        self.jump(addr + self.v(0) as u16);
    }

    fn call_addr(&mut self, addr: u16) {
        self.stack[self.sp as usize] = self.pc + 2;
        self.sp += 1;
        self.jump(addr);
    }

    fn se_vx_b(&mut self, x: u8, byte: u8) {
        self.skip_if(self.v(x) == byte);
    }

    fn se_vx_vy(&mut self, x: u8, y: u8) {
        self.skip_if(self.v(x) == self.v(y));
    }

    fn sne_vx_b(&mut self, x: u8, byte: u8) {
        self.skip_if(self.v(x) != byte);
    }

    fn sne_vx_vy(&mut self, x: u8, y: u8) {
        self.skip_if(self.v(x) != self.v(y));
    }

    fn ld_vx_b(&mut self, x: u8, value: u8) {
        self.write_register(x, value);
        self.increment();
    }

    fn ld_vx_vy(&mut self, x: u8, y: u8) {
        self.write_register(x, self.v(y));
        self.increment();
    }

    fn ld_i_addr(&mut self, addr: u16) {
        self.i = addr;
        self.increment();
    }

    fn ld_vx_dt(&mut self, x: u8) {
        self.write_register(x, self.dt as u8);
        self.increment();
    }

    fn ld_vx_k(&mut self, x: u8) {
        if let Some(key) = self.keyboard.get_pressed() {
            self.write_register(x, key);
            self.increment();
        }
    }

    fn ld_dt_vx(&mut self, x: u8) {
        self.dt = self.v(x) as u16;
        self.increment();
    }

    fn ld_st_vx(&mut self, x: u8) {
        self.st = self.v(x) as u16;
        self.increment();
    }

    fn ld_b_vx(&mut self, x: u8) {
        self.memory.write(self.i, self.v(x) / 100);
        self.memory.write(self.i + 1, (self.v(x) / 10) % 10);
        self.memory.write(self.i + 2, (self.v(x) % 100) % 10);
        self.increment();
    }

    fn ld_i_vx(&mut self, x: u8) {
        let x = x as usize;
        let i = self.i as usize;
        let slice = &self.v[0..x + 1];

        self.memory.write_slice(i, i + x + 1, slice);
        self.increment();
    }

    fn ld_vx_i(&mut self, x: u8) {
        let x = x as usize;
        let i = self.i as usize;
        let slice = self.memory.read_slice(i, i + x + 1);

        self.v[0..x + 1].copy_from_slice(slice);
        self.increment();
    }

    fn add_vx_b(&mut self, x: u8, rhs: u8) {
        self.write_register(x, self.v(x).wrapping_add(rhs));
        self.increment();
    }

    fn ld_f_vx(&mut self, x: u8) {
        self.i = x as u16 * 5;
        self.increment();
    }

    fn add_i_vx(&mut self, x: u8) {
        self.i = self.i.wrapping_add(self.v(x) as u16);
        self.increment();
    }

    fn add_vx_vy(&mut self, x: u8, y: u8) {
        let (result, carry) = self.v(x).overflowing_add(self.v(y));

        self.write_register(x, result);
        self.set_flag(carry);
        self.increment();
    }

    fn sub_vx_vy(&mut self, x: u8, y: u8) {
        let (result, borrow) = self.v(x).overflowing_sub(self.v(y));

        self.write_register(x, result);
        self.set_flag(!borrow);
        self.increment();
    }

    fn sub_vy_vx(&mut self, y: u8, x: u8) {
        let (result, borrow) = self.v(y).overflowing_sub(self.v(x));

        self.write_register(x, result);
        self.set_flag(!borrow);
        self.increment();
    }

    fn or_vx_vy(&mut self, x: u8, y: u8) {
        self.write_register(x, self.v(x) | self.v(y));
        self.increment();
    }

    fn and_vx_vy(&mut self, x: u8, y: u8) {
        self.write_register(x, self.v(x) & self.v(y));
        self.increment();
    }

    fn xor_vx_vy(&mut self, x: u8, y: u8) {
        self.write_register(x, self.v(x) ^ self.v(y));
        self.increment();
    }

    fn shr_vx(&mut self, x: u8) {
        let shift_flag = self.v(x) & 1 == 1;

        self.write_register(x, self.v(x).wrapping_shr(1));
        self.set_flag(shift_flag);
        self.increment();
    }

    fn shl_vx(&mut self, x: u8) {
        let shift_flag = self.v(x) >> 7 == 1;

        self.write_register(x, self.v(x).wrapping_shl(1));
        self.set_flag(shift_flag);
        self.increment();
    }

    fn rnd_vx_b(&mut self, x: u8, max: u8) {
        let value = self.rng.gen::<u8>() & max;

        self.write_register(x, value);
        self.increment();
    }

    fn drw_x_y_n(&mut self, x: u8, y: u8, n: u8) {
        let i = self.i as usize;
        let x = self.v(x) as usize;
        let y = self.v(y) as usize;
        let sprite = self.memory.read_slice(i, i + n as usize);
        let collision = self.display.load_sprite(x, y, sprite);

        self.set_flag(collision);
        self.increment();
    }

    fn skp_vx(&mut self, x: u8) {
        self.skip_if(self.keyboard.is_pressed(self.v(x)));
    }

    fn sknp_vx(&mut self, x: u8) {
        self.skip_if(!self.keyboard.is_pressed(self.v(x)));
    }

    fn write_register(&mut self, x: u8, value: u8) {
        self.v[x as usize] = value;
    }

    fn increment(&mut self) {
        self.pc += 2;
    }

    fn jump(&mut self, addr: u16) {
        self.pc = addr;
    }

    fn skip_if(&mut self, cond: bool) {
        self.pc += if cond { 4 } else { 2 };
    }

    fn set_flag(&mut self, cond: bool) {
        self.v[0xF] = if cond { 1 } else { 0 };
    }

    #[inline]
    fn v(&self, x: u8) -> u8 {
        self.v[x as usize]
    }
}
