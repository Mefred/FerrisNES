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
    flag_decimal: bool,
    flag_overflow: bool,
    flag_negative: bool,
    cycles: i32,
    stack_pointer: u8,
    address_bus: u16,
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
            flag_decimal: false,
            flag_overflow: false,
            flag_negative: false,
            cycles: 0,
            stack_pointer: 0,
            address_bus: 0,
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

    fn push(&mut self, value: u8) {
        self.write(0x0100 + self.stack_pointer as u16, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn pull(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.read(0x0100 + self.stack_pointer as u16)
    }

    fn fetch_byte(&mut self) -> u8 {
        let byte = self.read(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(1);
        byte
    }

    fn absolute_address(&mut self) {
        let low = self.fetch_byte() as u16;
        let high = self.fetch_byte() as u16;

        self.address_bus = (high << 8) | low;
    }

    fn asl(&mut self, address: u16, mut input: u8) {
        self.flag_carry = input >= 0x80;
        input <<= 1;
        self.write(address, input);
        self.flag_negative = input >= 0x80;
        self.flag_zero = input == 0x00;
    }

    fn lsr(&mut self, address: u16, mut input: u8) {
        self.flag_carry = (input & 0x01) != 0;
        input >>= 1;
        self.write(address, input);
        self.flag_negative = false;
        self.flag_zero = input == 0x00;
    }

    fn rol(&mut self, address: u16, mut input: u8) {
        let future_flag_carry = input >= 0x80;
        input <<= 1;
        if self.flag_carry {
            input |= 1;
        }
        self.write(address, input);
        self.flag_carry = future_flag_carry;
        self.flag_negative = input >= 0x80;
        self.flag_zero = input == 0x00;
    }

    fn ror(&mut self, address: u16, mut input: u8) {
        let future_flag_carry = (input & 0x01) != 0;
        input >>= 1;
        if self.flag_carry {
            input |= 0x80;
        }
        self.write(address, input);
        self.flag_carry = future_flag_carry;
        self.flag_negative = input >= 0x80;
        self.flag_zero = input == 0x00;
    }

    fn inc(&mut self, address: u16, mut input: u8) {
        input = input.wrapping_add(1);
        self.write(address, input);
        self.flag_negative = input >= 0x80;
        self.flag_zero = input == 0;
    }

    fn dec(&mut self, address: u16, mut input: u8) {
        input = input.wrapping_sub(1);
        self.write(address, input);
        self.flag_negative = input >= 0x80;
        self.flag_zero = input == 0;
    }

    pub fn reset(&mut self) {
        let pcl = self.read(0xFFFC);
        let pch = self.read(0xFFFD);

        self.program_counter = ((pch as u16) << 8) | (pcl as u16);
        self.flag_interrupt_disable = true;
        self.stack_pointer = 0xFD;
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
                0x48 => self.pha(),
                0x68 => self.pla(),
                0x20 => self.jsr(),
                0x60 => self.rts(),
                0x4C => self.jmp(),
                0xE8 => self.inx(),
                0xCA => self.dex(),
                0xC8 => self.iny(),
                0x88 => self.dey(),
                0xAA => self.tax(),
                0x8A => self.txa(),
                0xA8 => self.tay(),
                0x98 => self.tya(),
                0x9A => self.txs(),
                0xBA => self.tsx(),
                0x38 => self.sec(),
                0x18 => self.clc(),
                0xB8 => self.clv(),
                0x78 => self.sei(),
                0x58 => self.cli(),
                0xF8 => self.sed(),
                0xD8 => self.cld(),
                0xEA => self.cycles = 2,
                0x08 => self.php(),
                0x28 => self.plp(),
                0x0A => self.asl_a(),
                0x06 => self.asl_zero_page(),
                0x0E => self.asl_absolute(),
                0x2A => self.rol_a(),
                0x26 => self.rol_zero_page(),
                0x2E => self.rol_absolute(),
                0x4A => self.lsr_a(),
                0x46 => self.lsr_zero_page(),
                0x4E => self.lsr_absolute(),
                0x6A => self.ror_a(),
                0x66 => self.ror_zero_page(),
                0x6E => self.ror_absolute(),
                0xE6 => self.inc_zero_page(),
                0xEE => self.inc_absolute(),

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
    fn pha(&mut self) {
        self.push(self.register_a);

        self.cycles = 3;
    }
    fn pla(&mut self) {
        self.register_a = self.pull();

        self.flag_zero = self.register_a == 0;
        self.flag_negative = self.register_a >= 0x80;

        self.cycles = 4
    }
    fn jsr(&mut self) {
        let low = self.fetch_byte() as u16;
        let high = self.fetch_byte() as u16;
        self.push((self.program_counter / 256) as u8);
        self.push(self.program_counter as u8);
        self.program_counter = high * 256 + low;
        self.cycles = 6;
    }
    fn rts(&mut self) {
        let low = self.pull() as u16;
        let high = self.pull() as u16;
        self.program_counter = high * 256 + low;
        self.program_counter += 1;
        self.cycles = 6;
    }
    fn jmp(&mut self) {
        let low = self.fetch_byte() as u16;
        let high = self.fetch_byte() as u16;
        self.program_counter = high * 256 + low;
        self.cycles = 3;
    }
    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);

        self.cycles = 2;

        self.flag_zero = self.register_x == 0;
        self.flag_negative = self.register_x > 127;
    }
    fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);

        self.cycles = 2;

        self.flag_zero = self.register_x == 0;
        self.flag_negative = self.register_x > 127;
    }
    fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);

        self.cycles = 2;

        self.flag_zero = self.register_y == 0;
        self.flag_negative = self.register_y > 127;
    }
    fn dey(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);

        self.cycles = 2;

        self.flag_zero = self.register_y == 0;
        self.flag_negative = self.register_y > 127;
    }
    fn tax(&mut self) {
        self.register_x = self.register_a;

        self.cycles = 2;

        self.flag_zero = self.register_x == 0;
        self.flag_negative = self.register_x > 127;
    }
    fn txa(&mut self) {
        self.register_a = self.register_x;

        self.cycles = 2;

        self.flag_zero = self.register_a == 0;
        self.flag_negative = self.register_a > 127;
    }
    fn tay(&mut self) {
        self.register_y = self.register_a;

        self.cycles = 2;

        self.flag_zero = self.register_y == 0;
        self.flag_negative = self.register_y > 127;
    }
    fn tya(&mut self) {
        self.register_a = self.register_y;

        self.cycles = 2;

        self.flag_zero = self.register_a == 0;
        self.flag_negative = self.register_a > 127;
    }
    fn txs(&mut self) {
        self.stack_pointer = self.register_x;

        self.cycles = 2;
    }
    fn tsx(&mut self) {
        self.register_x = self.stack_pointer;

        self.cycles = 2;

        self.flag_zero = self.register_x == 0;
        self.flag_negative = self.register_x > 127;
    }
    fn sec(&mut self) {
        self.flag_carry = true;

        self.cycles = 2;
    }
    fn clc(&mut self) {
        self.flag_carry = false;

        self.cycles = 2;
    }
    fn clv(&mut self) {
        self.flag_overflow = false;

        self.cycles = 2;
    }
    fn sei(&mut self) {
        self.flag_interrupt_disable = true;

        self.cycles = 2;
    }
    fn cli(&mut self) {
        self.flag_interrupt_disable = false;

        self.cycles = 2;
    }
    fn sed(&mut self) {
        self.flag_decimal = true;

        self.cycles = 2;
    }
    fn cld(&mut self) {
        self.flag_decimal = false;

        self.cycles = 2;
    }
    fn php(&mut self) {
        let mut temp: u8 = 0;
        temp += if self.flag_carry { 1 } else { 0 };
        temp += if self.flag_zero { 2 } else { 0 };
        temp += if self.flag_interrupt_disable { 4 } else { 0 };
        temp += if self.flag_decimal { 8 } else { 0 };
        temp += 0x10;
        temp += 0x20;
        temp += if self.flag_overflow { 0x40 } else { 0 };
        temp += if self.flag_negative { 0x80 } else { 0 };
        self.push(temp);
        self.cycles = 3;
    }
    fn plp(&mut self) {
        let temp = self.pull();
        self.flag_carry = (temp & 1) != 0;
        self.flag_zero = (temp & 2) != 0;
        self.flag_interrupt_disable = (temp & 4) != 0;
        self.flag_decimal = (temp & 8) != 0;
        self.flag_overflow = (temp & 0x40) != 0;
        self.flag_negative = (temp & 0x80) != 0;
        self.cycles = 3;
    }
    fn asl_a(&mut self) {
        self.flag_carry = self.register_a > 127;
        self.register_a <<= 1;
        self.flag_zero = self.register_a == 0;
        self.flag_negative = self.register_a > 127;
        self.cycles = 2;
    }
    fn asl_zero_page(&mut self) {
        self.address_bus = self.fetch_byte() as u16;
        self.asl(self.address_bus, self.read(self.address_bus));
        self.cycles = 5;
    }
    fn asl_absolute(&mut self) {
        self.absolute_address();
        self.asl(self.address_bus, self.read(self.address_bus));
        self.cycles = 6;
    }
    fn rol_a(&mut self) {
        let future_flag_carry = self.register_a >= 0x80;
        self.register_a <<= 1;
        if self.flag_carry {
            self.register_a |= 1;
        }
        self.flag_carry = future_flag_carry;
        self.flag_negative = self.register_a >= 0x80;
        self.flag_zero = self.register_a == 0x00;
        self.cycles = 2;
    }
    fn rol_zero_page(&mut self) {
        self.address_bus = self.fetch_byte() as u16;
        self.rol(self.address_bus, self.read(self.address_bus));
        self.cycles = 5;
    }
    fn rol_absolute(&mut self) {
        self.absolute_address();
        self.rol(self.address_bus, self.read(self.address_bus));
        self.cycles = 6;
    }
    fn lsr_a(&mut self) {
        self.flag_carry = (self.register_a & 0x01) != 0;
        self.register_a >>= 1;
        self.flag_zero = self.register_a == 0;
        self.flag_negative = false;
        self.cycles = 2;
    }
    fn lsr_zero_page(&mut self) {
        self.address_bus = self.fetch_byte() as u16;
        self.lsr(self.address_bus, self.read(self.address_bus));
        self.cycles = 5;
    }
    fn lsr_absolute(&mut self) {
        self.absolute_address();
        self.lsr(self.address_bus, self.read(self.address_bus));
        self.cycles = 6;
    }
    fn ror_a(&mut self) {
        let future_flag_carry = (self.register_a & 0x01) != 0;
        self.register_a >>= 1;
        if self.flag_carry {
            self.register_a |= 0x80;
        }
        self.flag_carry = future_flag_carry;
        self.flag_negative = self.register_a >= 0x80;
        self.flag_zero = self.register_a == 0x00;
        self.cycles = 2;
    }
    fn ror_zero_page(&mut self) {
        self.address_bus = self.fetch_byte() as u16;
        self.ror(self.address_bus, self.read(self.address_bus));
        self.cycles = 5;
    }
    fn ror_absolute(&mut self) {
        self.absolute_address();
        self.ror(self.address_bus, self.read(self.address_bus));
        self.cycles = 6;
    }
    fn inc_zero_page(&mut self) {
        self.address_bus = self.fetch_byte() as u16;
        self.inc(self.address_bus, self.read(self.address_bus));
    }
    fn inc_absolute(&mut self) {
        self.absolute_address();
        self.inc(self.address_bus, self.read(self.address_bus));
    }
    // todo dec
}
