use std::fs;

pub struct CPU {
    program_counter: u16,
    register_a: u8,
    register_x: u8,
    register_y: u8,
    ram: [u8; 0x800],
    rom: [u8; 0x8000],
    flag_carry: bool,
    flag_zero: bool,
    flag_interrupt_disable: bool,
    dlag_decimal: bool,
    flag_overflow: bool,
    flag_negative: bool,
    cycles: i32,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            program_counter: 0,
            register_a: 0,
            register_x: 0,
            register_y: 0,
            ram: [0; 0x800],
            rom: [0; 0x8000],
            flag_carry: false,
            flag_zero: false,
            flag_interrupt_disable: false,
            dlag_decimal: false,
            flag_overflow: false,
            flag_negative: false,
            cycles: 0,
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        let rom_file = fs::read(path).unwrap();

        self.rom.copy_from_slice(&rom_file[0x10..0x10 + 0x8000]);
    }

    pub fn read_debug(&self, address: u16) -> u8 {
        self.read(address)
    }

    fn read(&self, address: u16) -> u8 {
        if address <= 0x1FFF {
            return self.ram[(address & 0x07FF) as usize];
        }
        if address >= 0x8000 {
            return self.rom[(address - 0x8000) as usize];
        }
        panic!("Unhandled address");
    }

    fn write(&mut self, address: u16, data: u8) {
        if address <= 0x1FFF {
            self.ram[(address & 0x07FF) as usize] = data;
        }
    }

    fn fetch_byte(&mut self) -> u8 {
        let byte = self.read(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(1);
        byte
    }

    pub fn reset(&mut self) {
        let pcl = self.read(0xFFFC);
        let pch = self.read(0xFFFD);

        self.program_counter = ((pch as u16) << 8) | (pcl as u16);
        self.flag_interrupt_disable = true;
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.read(self.program_counter);
            self.program_counter += 1;

            match opcode {
                0x02 => break,

                0xA0 => self.ldy_immediate(),

                0xA2 => self.ldx_immediate(),

                0xA9 => self.lda_immediate(),
                0xA5 => self.lda_zero_page(),
                0xAD => self.lda_absolute(),

                0x85 => self.sta_zero_page(),
                0x8D => self.sta_absolute(),

                0x86 => self.stx_zero_page(),
                0x8E => self.stx_absolute(),

                0x84 => self.sty_zero_page(),
                0x8C => self.sty_absolute(),

                0x10 => self.bpl(),
                0x30 => self.bmi(),
                0x50 => self.bvc(),
                0x70 => self.bvs(),
                0x90 => self.bcc(),
                0xB0 => self.bcs(),
                0xD0 => self.bne(),
                0xF0 => self.beq(),

                _ => todo!(),
            }
        }
    }

    fn ldy_immediate(&mut self) {
        self.register_y = self.fetch_byte();

        self.cycles = 2;

        self.flag_zero = self.register_y == 0;
        self.flag_negative = self.register_y > 127;
    }

    fn ldx_immediate(&mut self) {
        self.register_x = self.fetch_byte();

        self.cycles = 2;

        self.flag_zero = self.register_x == 0;
        self.flag_negative = self.register_x > 127;
    }

    fn lda_immediate(&mut self) {
        self.register_a = self.fetch_byte();

        self.cycles = 2;

        self.flag_zero = self.register_a == 0;
        self.flag_negative = self.register_a > 127;
    }

    fn lda_zero_page(&mut self) {
        let addr = self.fetch_byte() as u16;
        self.register_a = self.read(addr);

        self.cycles = 3;
    }

    fn lda_absolute(&mut self) {
        let low = self.fetch_byte() as u16;
        let high = self.fetch_byte() as u16;

        let addr = (high << 8) | low;

        self.register_a = self.read(addr);

        self.cycles = 4;
    }

    fn sta_zero_page(&mut self) {
        let addr = self.fetch_byte() as u16;
        self.write(addr, self.register_a);

        self.cycles = 3;
    }

    fn sta_absolute(&mut self) {
        let address_low = self.fetch_byte() as u16;
        let address_high = self.fetch_byte() as u16;
        self.write(address_high * 256 + address_low, self.register_a);

        self.cycles = 4;
    }

    fn stx_zero_page(&mut self) {
        let addr = self.fetch_byte() as u16;
        self.write(addr, self.register_x);

        self.cycles = 3;
    }

    fn stx_absolute(&mut self) {
        let address_low = self.fetch_byte() as u16;
        let address_high = self.fetch_byte() as u16;
        self.write(address_high * 256 + address_low, self.register_x);

        self.cycles = 4;
    }

    fn sty_zero_page(&mut self) {
        let addr = self.fetch_byte() as u16;
        self.write(addr, self.register_y);

        self.cycles = 3;
    }

    fn sty_absolute(&mut self) {
        let address_low = self.fetch_byte() as u16;
        let address_high = self.fetch_byte() as u16;
        self.write(address_high * 256 + address_low, self.register_y);

        self.cycles = 4;
    }

    fn bpl(&mut self) {
        let offset = self.fetch_byte() as i8;
        if !self.flag_negative {
            self.program_counter = self.program_counter.wrapping_add(offset as i16 as u16);
            self.cycles = 3;
        } else {
            self.cycles = 2
        }
    }

    fn bmi(&mut self) {
        let offset = self.fetch_byte() as i8;
        if self.flag_negative {
            self.program_counter = self.program_counter.wrapping_add(offset as i16 as u16);
            self.cycles = 3;
        } else {
            self.cycles = 2
        }
    }

    fn bvc(&mut self) {
        let offset = self.fetch_byte() as i8;
        if !self.flag_overflow {
            self.program_counter = self.program_counter.wrapping_add(offset as i16 as u16);
            self.cycles = 3;
        } else {
            self.cycles = 2
        }
    }

    fn bvs(&mut self) {
        let offset = self.fetch_byte() as i8;
        if self.flag_overflow {
            self.program_counter = self.program_counter.wrapping_add(offset as i16 as u16);
            self.cycles = 3;
        } else {
            self.cycles = 2
        }
    }

    fn bcc(&mut self) {
        let offset = self.fetch_byte() as i8;
        if !self.flag_carry {
            self.program_counter = self.program_counter.wrapping_add(offset as i16 as u16);
            self.cycles = 3;
        } else {
            self.cycles = 2
        }
    }

    fn bcs(&mut self) {
        let offset = self.fetch_byte() as i8;
        if self.flag_carry {
            self.program_counter = self.program_counter.wrapping_add(offset as i16 as u16);
            self.cycles = 3;
        } else {
            self.cycles = 2
        }
    }

    fn bne(&mut self) {
        let offset = self.fetch_byte() as i8;
        if !self.flag_zero {
            self.program_counter = self.program_counter.wrapping_add(offset as i16 as u16);
            self.cycles = 3;
        } else {
            self.cycles = 2
        }
    }

    fn beq(&mut self) {
        let offset = self.fetch_byte() as i8;
        if self.flag_zero {
            self.program_counter = self.program_counter.wrapping_add(offset as i16 as u16);
            self.cycles = 3;
        } else {
            self.cycles = 2
        }
    }
}
