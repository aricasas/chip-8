#[derive(Debug)]
pub struct ComputerState {
    memory: [u8; 4096],
    pub display: [u64; 32],
    pc: u16,
    idx: u16,
    sp: u16,
    delay_timer: u8,
    sound_timer: u8,
    regs: [u8; 16],
    pressed_keys: u16,
}

impl ComputerState {
    pub fn new(memory: &[u8]) -> Box<Self> {
        let mut state = Box::new(Self {
            memory: [0; 4096],
            display: [0; 32],
            pc: 512,
            idx: 0,
            sp: 4096 - 256,
            delay_timer: 0,
            sound_timer: 0,
            regs: [0; 16],
            pressed_keys: 0,
        });

        state.memory[0..80].copy_from_slice(&FONT);
        for (i, &x) in memory.iter().enumerate() {
            state.memory[512 + i] = x;
        }

        state
    }

    pub fn is_beeping(&self) -> bool {
        self.sound_timer > 0
    }

    pub fn set_pressed_keys(&mut self, keys: u16) {
        self.pressed_keys = keys;
    }

    /// Advances computer state 1/60th of a second
    pub fn advance_tick(&mut self, cpu_cycles: u32) {
        for _ in 0..cpu_cycles {
            self.cpu_cycle();
        }
        self.delay_timer_cycle();
        self.sound_timer_cycle();
    }

    fn cpu_cycle(&mut self) {
        // Fetch
        let instruction: u16 = ((self.memory[self.pc as usize] as u16) << 8)
            | ((self.memory[self.pc as usize + 1]) as u16);
        let nibbles: [u8; 4] = [
            (instruction >> 12 & 0xF) as u8,
            (instruction >> 8 & 0xF) as u8,
            (instruction >> 4 & 0xF) as u8,
            (instruction & 0xF) as u8,
        ];
        self.pc += 2;

        // Decode
        let X = nibbles[1] as usize;
        let Y = nibbles[2] as usize;
        let N = (instruction & 0xF) as u8;
        let NN = (instruction & 0xFF) as u8;
        let NNN = instruction & 0xFFF;

        // Execute
        match nibbles {
            [0x0, 0x0, 0xE, 0x0] => {
                // Clear screen
                self.display = [0; 32];
            }
            [0x0, 0x0, 0xE, 0xE] => {
                // Return from subroutine
                self.pc = (self.memory[self.sp as usize] as u16) << 8;
                self.pc |= self.memory[self.sp as usize + 1] as u16;
                self.sp += 2;
            }
            [0x0, _, _, _] => {
                // Execute machine language routine
                // Nothing
            }
            [0x1, _, _, _] => {
                // Jump to NNN
                self.pc = NNN;
            }
            [0x2, _, _, _] => {
                // Jump to subroutine NNN
                self.sp -= 2;
                self.memory[self.sp as usize] = (self.pc >> 8) as u8;
                self.memory[self.sp as usize + 1] = (self.pc & 0xFF) as u8;
                self.pc = NNN;
            }
            [0x3, _, _, _] => {
                // Conditional skip if equals VX==NN
                if self.regs[X] == NN {
                    self.pc += 2;
                }
            }
            [0x4, _, _, _] => {
                // Conditional skip if not equals VX!=NN
                if self.regs[X] != NN {
                    self.pc += 2;
                }
            }
            [0x5, _, _, 0x0] => {
                // Conditional skip if equal VX==VY
                if self.regs[X] == self.regs[Y] {
                    self.pc += 2;
                }
            }
            [0x6, _, _, _] => {
                // Set VX=NN
                self.regs[X] = NN;
            }
            [0x7, _, _, _] => {
                // Add VX+=NN
                self.regs[X] = self.regs[X].wrapping_add(NN);
            }
            [0x8, _, _, 0x0] => {
                // Set VX=VY
                self.regs[X] = self.regs[Y];
            }
            [0x8, _, _, 0x1] => {
                // Binary OR VX=VX|VY
                self.regs[X] |= self.regs[Y];
            }
            [0x8, _, _, 0x2] => {
                // Binary AND VX=VX|VY
                self.regs[X] &= self.regs[Y];
            }
            [0x8, _, _, 0x3] => {
                // Binary XOR VX=VX^VY
                self.regs[X] ^= self.regs[Y];
            }
            [0x8, _, _, 0x4] => {
                // Add VX+=VY
                let (sum, overflow) = self.regs[X].overflowing_add(self.regs[Y]);
                self.regs[X] = sum;
                self.regs[0xF] = if overflow { 1 } else { 0 };
            }
            [0x8, _, _, 0x5] => {
                // Subtract VX-=VY
                let (sum, overflow) = self.regs[X].overflowing_sub(self.regs[Y]);
                self.regs[X] = sum;
                self.regs[0xF] = if overflow { 0 } else { 1 };
            }
            [0x8, _, _, 0x6] => {
                // Shift right VX>>=1
                let shifted_bit = self.regs[X] & 1;
                self.regs[X] >>= 1;
                self.regs[0xF] = shifted_bit;
            }
            [0x8, _, _, 0x7] => {
                // Subtract VX=VY-VX
                let (sum, overflow) = self.regs[Y].overflowing_sub(self.regs[X]);
                self.regs[X] = sum;
                self.regs[0xF] = if overflow { 0 } else { 1 };
            }
            [0x8, _, _, 0xE] => {
                // Shift left VX<<=1
                let shifted_bit = self.regs[X] >> 7 & 1;
                self.regs[X] <<= 1;
                self.regs[0xF] = shifted_bit;
            }
            [0x9, _, _, 0x0] => {
                // Conditional skip if not equal VX!=VY
                if self.regs[X] != self.regs[Y] {
                    self.pc += 2;
                }
            }
            [0xA, _, _, _] => {
                // Set index I=NNN
                self.idx = NNN;
            }
            [0xB, _, _, _] => {
                // Jump with offset to V0+NNN
                self.pc = self.regs[0x0] as u16 + NNN;
            }
            [0xC, _, _, _] => {
                // Set random bitwise AND VX=rand()&NN
                self.regs[X] = fastrand::u8(..) & NN;
            }
            [0xD, _, _, _] => {
                // Display sprite
                let x_coord = self.regs[X] % 64;
                let y_coord = self.regs[Y] % 32;
                self.regs[0xF] = 0;

                for (i, row) in (y_coord as usize..(32.min(y_coord + N) as usize)).enumerate() {
                    let sprite_row = self.memory[self.idx as usize + i];
                    let mask = ((sprite_row as u64) << 56) >> x_coord;
                    if (self.display[row] & mask) != 0 {
                        self.regs[0xF] = 1;
                    }
                    self.display[row] ^= mask;
                }
            }
            [0xE, _, 0x9, 0xE] => {
                // Conditional skip if key VX pressed
                if self.pressed_keys >> self.regs[X] & 1 == 1 {
                    self.pc += 2;
                }
            }
            [0xE, _, 0xA, 0x1] => {
                // Conditional skip if not key VX pressed
                if self.pressed_keys >> self.regs[X] & 1 == 0 {
                    self.pc += 2;
                }
            }
            [0xF, _, 0x0, 0x7] => {
                // Set VX to delay timer VX=timer()
                self.regs[X] = self.delay_timer;
            }
            [0xF, _, 0x1, 0x5] => {
                // Set delay timer to VX timer()=VX
                self.delay_timer = self.regs[X];
            }
            [0xF, _, 0x1, 0x8] => {
                // Set sound timer to VX timer()=VX
                self.sound_timer = self.regs[X];
            }
            [0xF, _, 0x1, 0xE] => {
                // Add to index register I+=VX
                self.idx = self.idx.wrapping_add(self.regs[X] as u16);
            }
            [0xF, _, 0x0, 0xA] => {
                // Block and get key
                if self.pressed_keys == 0 {
                    self.pc -= 2;
                } else {
                    self.regs[X] = self.pressed_keys.trailing_zeros() as u8;
                }
            }
            [0xF, _, 0x2, 0x9] => {
                // Font character I=font(hex(VX last nibble))
                self.idx = 5 * (self.regs[X] & 0xF) as u16;
            }
            [0xF, _, 0x3, 0x3] => {
                // Binary coded decimal VX [I]=100s digit, [I+1]=10s digit, [I+2]=1s digit
                self.memory[self.idx as usize] = self.regs[X] / 100;
                self.memory[self.idx as usize + 1] = (self.regs[X] % 100) / 10;
                self.memory[self.idx as usize + 2] = self.regs[X] % 10;
            }
            [0xF, _, 0x5, 0x5] => {
                // Store registers
                for i in 0..=X {
                    self.memory[self.idx as usize + i] = self.regs[i];
                }
            }
            [0xF, _, 0x6, 0x5] => {
                // Load registers
                for i in 0..=X {
                    self.regs[i] = self.memory[self.idx as usize + i];
                }
            }
            _ => {
                println!(
                    "(PC={:#x}) Invalid Instruction: {:02x}",
                    self.pc, instruction
                );
            }
        }
    }
    fn delay_timer_cycle(&mut self) {
        self.delay_timer = self.delay_timer.saturating_sub(1);
    }
    fn sound_timer_cycle(&mut self) {
        self.sound_timer = self.sound_timer.saturating_sub(1);
    }
}

const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];
