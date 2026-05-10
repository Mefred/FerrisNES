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

const OPCODE_NAMES: [&str; 256] = [
    "BRK", "ORA", "HLT", "SLO", "NOP", "ORA", "ASL", "SLO", "PHP", "ORA", "ASL", "ANC", "NOP",
    "ORA", "ASL", "SLO", "BPL", "ORA", "HLT", "SLO", "NOP", "ORA", "ASL", "SLO", "CLC", "ORA",
    "NOP", "SLO", "NOP", "ORA", "ASL", "SLO", "JSR", "AND", "HLT", "RLA", "BIT", "AND", "ROL",
    "RLA", "PLP", "AND", "ROL", "ANC", "BIT", "AND", "ROL", "RLA", "BMI", "AND", "HLT", "RLA",
    "NOP", "AND", "ROL", "RLA", "SEC", "AND", "NOP", "RLA", "NOP", "AND", "ROL", "RLA", "RTI",
    "EOR", "HLT", "SRE", "NOP", "EOR", "LSR", "SRE", "PHA", "EOR", "LSR", "ALR", "JMP", "EOR",
    "LSR", "SRE", "BVC", "EOR", "HLT", "SRE", "NOP", "EOR", "LSR", "SRE", "CLI", "EOR", "NOP",
    "SRE", "NOP", "EOR", "LSR", "SRE", "RTS", "ADC", "HLT", "RRA", "NOP", "ADC", "ROR", "RRA",
    "PLA", "ADC", "ROR", "ARR", "JMP", "ADC", "ROR", "RRA", "BVS", "ADC", "HLT", "RRA", "NOP",
    "ADC", "ROR", "RRA", "SEI", "ADC", "NOP", "RRA", "NOP", "ADC", "ROR", "RRA", "NOP", "STA",
    "NOP", "SAX", "STY", "STA", "STX", "SAX", "DEY", "NOP", "TXA", "ANE", "STY", "STA", "STX",
    "SAX", "BCC", "STA", "HLT", "SHA", "STY", "STA", "STX", "SAX", "TYA", "STA", "TXS", "SHS",
    "SHY", "STA", "SHX", "SHA", "LDY", "LDA", "LDX", "LAX", "LDY", "LDA", "LDX", "LAX", "TAY",
    "LDA", "TAX", "LXA", "LDY", "LDA", "LDX", "LAX", "BCS", "LDA", "HLT", "LAX", "LDY", "LDA",
    "LDX", "LAX", "CLV", "LDA", "TSX", "LAE", "LDY", "LDA", "LDX", "LAX", "CPY", "CMP", "NOP",
    "DCP", "CPY", "CMP", "DEC", "DCP", "INY", "CMP", "DEX", "AXS", "CPY", "CMP", "DEC", "DCP",
    "BNE", "CMP", "HLT", "DCP", "NOP", "CMP", "DEC", "DPC", "CLD", "CMP", "NOP", "DCP", "NOP",
    "CMP", "DEC", "DCP", "CPX", "SBC", "NOP", "ISC", "CPX", "SBC", "INC", "ISC", "INX", "SBC",
    "NOP", "SBC", "CPX", "SBC", "INC", "ISC", "BEQ", "SBC", "HLT", "ISC", "NOP", "SBC", "INC",
    "ISC", "SED", "SBC", "NOP", "ISC", "NOP", "SBC", "INC", "ISC",
];

const LOGGING: bool = false;

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

    fn trace_logger(&self, opcode: u8) {
        if !LOGGING {
            return;
        }

        let pc = self.program_counter;

        let op = self.read_debug(pc);
        let b1 = self.read_debug(pc.wrapping_add(1));
        let b2 = self.read_debug(pc.wrapping_add(2));

        let opcode_name = OPCODE_NAMES[opcode as usize];

        let flags = format!(
            "{}{}{}{}{}{}",
            if self.flag_negative { "N" } else { "." },
            if self.flag_overflow { "V" } else { "." },
            if self.flag_decimal { "D" } else { "." },
            if self.flag_interrupt_disable {
                "I"
            } else {
                "."
            },
            if self.flag_zero { "Z" } else { "." },
            if self.flag_carry { "C" } else { "." },
        );

        println!(
            "{:04X}  {:02X} {:02X} {:02X}  {:<4}  A:{:02X} X:{:02X} Y:{:02X} P:{} SP:{:02X}",
            pc,
            op,
            b1,
            b2,
            opcode_name,
            self.register_a,
            self.register_x,
            self.register_y,
            flags,
            self.stack_pointer
        );
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

    fn absolute_address_x_indexed(&mut self) {
        self.address_bus = self.fetch_byte() as u16;
        self.address_bus = (self.fetch_byte() as u16) << 8 | self.address_bus;
        self.address_bus = self.address_bus.wrapping_add(self.register_x as u16);
    }

    fn absolute_address_y_indexed(&mut self) {
        self.address_bus = self.fetch_byte() as u16;
        self.address_bus = (self.fetch_byte() as u16) << 8 | self.address_bus;
        self.address_bus = self.address_bus.wrapping_add(self.register_y as u16);
    }

    fn zero_page_address_x_indexed(&mut self) {
        let addr = self.fetch_byte();
        self.address_bus = addr.wrapping_add(self.register_x) as u16;
    }

    fn zero_page_address_y_indexed(&mut self) {
        let addr = self.fetch_byte();
        self.address_bus = addr.wrapping_add(self.register_y) as u16;
    }

    fn indirect_address_y_indexed(&mut self) {
        let addr = self.fetch_byte();

        let low = self.read(addr as u16) as u16;
        let high = self.read(addr.wrapping_add(1) as u16) as u16;

        self.address_bus = (high << 8) | low;
        self.address_bus = self.address_bus.wrapping_add(self.register_y as u16);
    }

    fn indirect_address_x_indexed(&mut self) {
        let addr = self.fetch_byte() + self.register_x;

        let low = self.read(addr as u16) as u16;
        let high = self.read(addr.wrapping_add(1) as u16) as u16;

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

    fn ora(&mut self, input: u8) {
        self.register_a |= input;
        self.flag_negative = self.register_a >= 0x80;
        self.flag_zero = self.register_a == 0;
    }

    fn and(&mut self, input: u8) {
        self.register_a &= input;
        self.flag_negative = self.register_a >= 0x80;
        self.flag_zero = self.register_a == 0;
    }

    fn eor(&mut self, input: u8) {
        self.register_a ^= input;
        self.flag_negative = self.register_a >= 0x80;
        self.flag_zero = self.register_a == 0;
    }

    fn adc(&mut self, input: u8) {
        let int_sum = input as u16 + self.register_a as u16 + if self.flag_carry { 1 } else { 0 };
        self.flag_overflow =
            (!(self.register_a ^ input) & (self.register_a ^ int_sum as u8) & 0x80) != 0;
        self.flag_carry = int_sum > 0xFF;
        self.register_a = int_sum as u8;
        self.flag_negative = (self.register_a & 0x80) != 0;
        self.flag_zero = self.register_a == 0;
    }

    fn sbc(&mut self, input: u8) {
        let borrow = if self.flag_carry { 0 } else { 1 };
        let int_sum = self.register_a as i16 - input as i16 - borrow as i16;
        self.flag_overflow =
            ((self.register_a ^ input) & (self.register_a ^ int_sum as u8) & 0x80) != 0;
        self.flag_carry = int_sum >= 0;
        self.register_a = int_sum as u8;
        self.flag_negative = (self.register_a & 0x80) != 0;
        self.flag_zero = self.register_a == 0;
    }

    fn cmp(&mut self, input: u8) {
        self.flag_carry = input <= self.register_a;
        self.flag_zero = input == self.register_a;
        self.flag_negative = self.register_a.wrapping_sub(input) > 127;
    }

    fn cpx(&mut self, input: u8) {
        self.flag_carry = input <= self.register_x;
        self.flag_zero = input == self.register_x;
        self.flag_negative = self.register_x.wrapping_sub(input) > 127;
    }

    fn cpy(&mut self, input: u8) {
        self.flag_carry = input <= self.register_y;
        self.flag_zero = input == self.register_y;
        self.flag_negative = self.register_y.wrapping_sub(input) > 127;
    }

    fn bit(&mut self, input: u8) {
        self.flag_zero = (self.register_a & input) == 0;
        self.flag_negative = (input & 0x80) != 0;
        self.flag_overflow = (input & 0x40) != 0;
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
            self.trace_logger(opcode);
            self.program_counter += 1;

            match opcode {
                0x02 => break,

                0xA0 => self.ldy_immediate(),
                0xA4 => self.ldy_zero_page(),
                0xB4 => self.ldy_zero_page_x(),
                0xAC => self.ldy_absolute(),
                0xBC => self.ldy_absolute_x(),

                0xA2 => self.ldx_immediate(),
                0xA6 => self.ldx_zero_page(),
                0xB6 => self.ldx_zero_page_y(),
                0xAE => self.ldx_absolute(),
                0xBE => self.ldx_absolute_y(),

                0xA9 => self.lda_immediate(),
                0xA5 => self.lda_zero_page(),
                0xAD => self.lda_absolute(),
                0xB5 => self.lda_zero_page_x(),
                0xBD => self.lda_absolute_x(),
                0xB9 => self.lda_absolute_y(),
                0xA1 => self.lda_indirect_x(),
                0xB1 => self.lda_indirect_y(),

                0x85 => self.sta_zero_page(),
                0x95 => self.sta_zero_page_x(),
                0x8D => self.sta_absolute(),
                0x9D => self.sta_absolute_x(),
                0x99 => self.sta_absolute_y(),
                0x81 => self.sta_indirect_x(),
                0x91 => self.sta_indirect_y(),

                0x86 => self.stx_zero_page(),
                0x96 => self.stx_zero_page_y(),
                0x8E => self.stx_absolute(),

                0x84 => self.sty_zero_page(),
                0x94 => self.sty_zero_page_x(),
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

                0x20 => self.jsr_absolute(),

                0x60 => self.rts_implied(),

                0x4C => self.jmp_absoute(),
                0x6C => self.jmp_indirect(),

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

                0x0A => self.asl_accumulator(),
                0x06 => self.asl_zero_page(),
                0x16 => self.asl_zero_page_x(),
                0x0E => self.asl_absolute(),
                0x1E => self.asl_absolute_x(),

                0x2A => self.rol_accumulator(),
                0x26 => self.rol_zero_page(),
                0x36 => self.rol_zero_page_x(),
                0x2E => self.rol_absolute(),
                0x3E => self.rol_absolute_x(),

                0x4A => self.lsr_accumulator(),
                0x46 => self.lsr_zero_page(),
                0x56 => self.lsr_zero_page_x(),
                0x4E => self.lsr_absolute(),
                0x5E => self.lsr_absolute_x(),

                0x6A => self.ror_accumulator(),
                0x66 => self.ror_zero_page(),
                0x76 => self.ror_zero_page_x(),
                0x6E => self.ror_absolute(),
                0x7E => self.ror_absolute_x(),

                0xE6 => self.inc_zero_page(),
                0xF6 => self.inc_zero_page_x(),
                0xEE => self.inc_absolute(),
                0xFE => self.inc_absolute_x(),

                0xC6 => self.dec_zero_page(),
                0xD6 => self.dec_zero_page_x(),
                0xCE => self.dec_absolute(),
                0xDE => self.dec_absolute_x(),

                0x09 => self.ora_immediate(),
                0x05 => self.ora_zero_page(),
                0x15 => self.ora_zero_page_x(),
                0x0D => self.ora_absolute(),
                0x1D => self.ora_absolute_x(),
                0x19 => self.ora_absolute_y(),
                0x01 => self.ora_indirect_x(),
                0x11 => self.ora_indirect_y(),

                0x29 => self.and_immediate(),
                0x25 => self.and_zero_page(),
                0x35 => self.and_zero_page_x(),
                0x2D => self.and_absolute(),
                0x3D => self.and_absolute_x(),
                0x39 => self.and_absolute_y(),
                0x21 => self.and_indirect_x(),
                0x31 => self.and_indirect_y(),

                0x49 => self.eor_immediate(),
                0x45 => self.eor_zero_page(),
                0x55 => self.eor_zero_page_x(),
                0x4D => self.eor_absolute(),
                0x5D => self.eor_absolute_x(),
                0x59 => self.eor_absolute_y(),
                0x41 => self.eor_indirect_x(),
                0x51 => self.eor_indirect_y(),

                0x69 => self.adc_immediate(),
                0x65 => self.adc_zero_page(),
                0x75 => self.adc_zero_page_x(),
                0x6D => self.adc_absolute(),
                0x7D => self.adc_absolute_x(),
                0x79 => self.adc_absolute_y(),
                0x61 => self.adc_indirect_x(),
                0x71 => self.adc_indirect_y(),

                0xE9 => self.sbc_immediate(),
                0xE5 => self.sbc_zero_page(),
                0xF5 => self.sbc_zero_page_x(),
                0xED => self.sbc_absolute(),
                0xFD => self.sbc_absolute_x(),
                0xF9 => self.sbc_absolute_y(),
                0xE1 => self.sbc_indirect_x(),
                0xF1 => self.sbc_indirect_y(),

                0xC9 => self.cmp_immediate(),
                0xC5 => self.cmp_zero_page(),
                0xD5 => self.cmp_zero_page_x(),
                0xCD => self.cmp_absolute(),
                0xDD => self.cmp_absolute_x(),
                0xD9 => self.cmp_absolute_y(),
                0xC1 => self.cmp_indirect_x(),
                0xD1 => self.cmp_indirect_y(),

                0xE0 => self.cpx_immediate(),
                0xE4 => self.cpx_zero_page(),
                0xEC => self.cpx_absolute(),

                0xC0 => self.cpy_immediate(),
                0xC4 => self.cpy_zero_page(),
                0xCC => self.cpy_absolute(),

                0x00 => self.brk(),

                0x40 => self.rti(),

                0x24 => self.bit_zero_page(),
                0x2C => self.bit_absolute(),

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
    fn ldy_zero_page(&mut self) {
        self.address_bus = self.fetch_byte() as u16;

        self.register_y = self.read(self.address_bus);

        self.cycles = 3;
    }
    fn ldy_zero_page_x(&mut self) {
        self.zero_page_address_x_indexed();

        self.register_y = self.read(self.address_bus);

        self.cycles = 4;
    }
    fn ldy_absolute(&mut self) {
        self.absolute_address();

        self.register_y = self.read(self.address_bus);

        self.cycles = 4;
    }
    fn ldy_absolute_x(&mut self) {
        self.absolute_address_x_indexed();

        self.register_y = self.read(self.address_bus);

        self.cycles = 4;
    }

    fn ldx_immediate(&mut self) {
        self.register_x = self.fetch_byte();

        self.cycles = 2;

        self.flag_zero = self.register_x == 0;
        self.flag_negative = self.register_x > 127;
    }
    fn ldx_zero_page(&mut self) {
        self.address_bus = self.fetch_byte() as u16;

        self.register_x = self.read(self.address_bus);

        self.cycles = 3;
    }
    fn ldx_zero_page_y(&mut self) {
        self.zero_page_address_y_indexed();

        self.register_x = self.read(self.address_bus);

        self.cycles = 4;
    }
    fn ldx_absolute(&mut self) {
        self.absolute_address();

        self.register_x = self.read(self.address_bus);

        self.cycles = 4;
    }
    fn ldx_absolute_y(&mut self) {
        self.absolute_address_y_indexed();

        self.register_x = self.read(self.address_bus);

        self.cycles = 4;
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
    fn lda_zero_page_x(&mut self) {
        self.zero_page_address_x_indexed();

        self.register_a = self.read(self.address_bus);

        self.cycles = 4;
    }
    fn lda_absolute(&mut self) {
        let low = self.fetch_byte() as u16;
        let high = self.fetch_byte() as u16;

        let addr = (high << 8) | low;

        self.register_a = self.read(addr);

        self.cycles = 4;
    }
    fn lda_absolute_x(&mut self) {
        self.absolute_address_x_indexed();

        self.register_a = self.read(self.address_bus);

        self.cycles = 4;
    }
    fn lda_absolute_y(&mut self) {
        self.absolute_address_y_indexed();

        self.register_a = self.read(self.address_bus);

        self.cycles = 4;
    }
    fn lda_indirect_x(&mut self) {
        self.indirect_address_x_indexed();

        self.register_a = self.read(self.address_bus);

        self.cycles = 6;
    }
    fn lda_indirect_y(&mut self) {
        self.indirect_address_y_indexed();

        self.register_a = self.read(self.address_bus);

        self.cycles = 5;
    }

    fn sta_zero_page(&mut self) {
        let addr = self.fetch_byte() as u16;
        self.write(addr, self.register_a);

        self.cycles = 3;
    }
    fn sta_zero_page_x(&mut self) {
        self.zero_page_address_x_indexed();
        self.write(self.address_bus, self.register_a);

        self.cycles = 4;
    }
    fn sta_absolute(&mut self) {
        let address_low = self.fetch_byte() as u16;
        let address_high = self.fetch_byte() as u16;
        self.write(address_high * 256 + address_low, self.register_a);

        self.cycles = 4;
    }
    fn sta_absolute_x(&mut self) {
        self.absolute_address_x_indexed();
        self.write(self.address_bus, self.register_a);

        self.cycles = 4;
    }
    fn sta_absolute_y(&mut self) {
        self.absolute_address_y_indexed();
        self.write(self.address_bus, self.register_a);

        self.cycles = 4;
    }
    fn sta_indirect_x(&mut self) {
        self.indirect_address_x_indexed();
        self.write(self.address_bus, self.register_a);

        self.cycles = 6;
    }
    fn sta_indirect_y(&mut self) {
        self.indirect_address_y_indexed();
        self.write(self.address_bus, self.register_a);

        self.cycles = 5;
    }

    fn stx_zero_page(&mut self) {
        let addr = self.fetch_byte() as u16;
        self.write(addr, self.register_x);

        self.cycles = 3;
    }
    fn stx_zero_page_y(&mut self) {
        self.zero_page_address_y_indexed();
        self.write(self.address_bus, self.register_x);

        self.cycles = 4;
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
    fn sty_zero_page_x(&mut self) {
        self.zero_page_address_x_indexed();
        self.write(self.address_bus, self.register_y);

        self.cycles = 4;
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

    fn jsr_absolute(&mut self) {
        let low = self.fetch_byte() as u16;
        let high = self.fetch_byte() as u16;
        self.push((self.program_counter / 256) as u8);
        self.push(self.program_counter as u8);
        self.program_counter = high * 256 + low;
        self.cycles = 6;
    }

    fn rts_implied(&mut self) {
        let low = self.pull() as u16;
        let high = self.pull() as u16;
        self.program_counter = high * 256 + low;
        self.program_counter += 1;
        self.cycles = 6;
    }

    fn jmp_absoute(&mut self) {
        let low = self.fetch_byte() as u16;
        let high = self.fetch_byte() as u16;
        self.program_counter = high * 256 + low;
        self.cycles = 3;
    }
    fn jmp_indirect(&mut self) {
        let low = self.fetch_byte() as u16;
        let high = self.fetch_byte() as u16;

        let pointer = (high << 8) | low;

        let target_low = self.read(pointer) as u16;

        let high_addr = if (pointer & 0x00FF) == 0x00FF {
            pointer & 0xFF00
        } else {
            pointer + 1
        };

        let target_high = self.read(high_addr) as u16;

        self.program_counter = (target_high << 8) | target_low;

        self.cycles = 5;
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

    fn asl_accumulator(&mut self) {
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
    fn asl_zero_page_x(&mut self) {
        self.zero_page_address_x_indexed();
        self.asl(self.address_bus, self.read(self.address_bus));
        self.cycles = 6;
    }
    fn asl_absolute(&mut self) {
        self.absolute_address();
        self.asl(self.address_bus, self.read(self.address_bus));
        self.cycles = 6;
    }
    fn asl_absolute_x(&mut self) {
        self.absolute_address_x_indexed();
        self.asl(self.address_bus, self.read(self.address_bus));
        self.cycles = 7;
    }

    fn rol_accumulator(&mut self) {
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
    fn rol_zero_page_x(&mut self) {
        self.zero_page_address_x_indexed();
        self.rol(self.address_bus, self.read(self.address_bus));
        self.cycles = 6;
    }
    fn rol_absolute(&mut self) {
        self.absolute_address();
        self.rol(self.address_bus, self.read(self.address_bus));
        self.cycles = 6;
    }
    fn rol_absolute_x(&mut self) {
        self.absolute_address_x_indexed();
        self.rol(self.address_bus, self.read(self.address_bus));
        self.cycles = 7;
    }

    fn lsr_accumulator(&mut self) {
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
    fn lsr_zero_page_x(&mut self) {
        self.zero_page_address_x_indexed();
        self.lsr(self.address_bus, self.read(self.address_bus));
        self.cycles = 6;
    }
    fn lsr_absolute(&mut self) {
        self.absolute_address();
        self.lsr(self.address_bus, self.read(self.address_bus));
        self.cycles = 6;
    }
    fn lsr_absolute_x(&mut self) {
        self.absolute_address_x_indexed();
        self.lsr(self.address_bus, self.read(self.address_bus));
        self.cycles = 7;
    }

    fn ror_accumulator(&mut self) {
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
    fn ror_zero_page_x(&mut self) {
        self.zero_page_address_x_indexed();
        self.ror(self.address_bus, self.read(self.address_bus));
        self.cycles = 6;
    }
    fn ror_absolute(&mut self) {
        self.absolute_address();
        self.ror(self.address_bus, self.read(self.address_bus));
        self.cycles = 6;
    }
    fn ror_absolute_x(&mut self) {
        self.absolute_address_x_indexed();
        self.ror(self.address_bus, self.read(self.address_bus));
        self.cycles = 7;
    }

    fn inc_zero_page(&mut self) {
        self.address_bus = self.fetch_byte() as u16;
        self.inc(self.address_bus, self.read(self.address_bus));
        self.cycles = 5;
    }
    fn inc_zero_page_x(&mut self) {
        self.zero_page_address_x_indexed();
        self.inc(self.address_bus, self.read(self.address_bus));
        self.cycles = 6;
    }
    fn inc_absolute(&mut self) {
        self.absolute_address();
        self.inc(self.address_bus, self.read(self.address_bus));
        self.cycles = 6;
    }
    fn inc_absolute_x(&mut self) {
        self.absolute_address_x_indexed();
        self.inc(self.address_bus, self.read(self.address_bus));
        self.cycles = 7;
    }

    fn dec_zero_page(&mut self) {
        self.address_bus = self.fetch_byte() as u16;
        self.dec(self.address_bus, self.read(self.address_bus));
        self.cycles = 5;
    }
    fn dec_zero_page_x(&mut self) {
        self.zero_page_address_x_indexed();
        self.dec(self.address_bus, self.read(self.address_bus));
        self.cycles = 6;
    }
    fn dec_absolute(&mut self) {
        self.absolute_address();
        self.dec(self.address_bus, self.read(self.address_bus));
        self.cycles = 6;
    }
    fn dec_absolute_x(&mut self) {
        self.absolute_address_x_indexed();
        self.dec(self.address_bus, self.read(self.address_bus));
        self.cycles = 7;
    }

    fn ora_immediate(&mut self) {
        let input = self.fetch_byte();
        self.ora(input);
        self.cycles = 2;
    }
    fn ora_zero_page(&mut self) {
        let input = self.fetch_byte() as u16;
        self.ora(self.read(input));
        self.cycles = 3;
    }
    fn ora_zero_page_x(&mut self) {
        self.zero_page_address_x_indexed();
        self.ora(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn ora_absolute(&mut self) {
        self.absolute_address();
        self.ora(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn ora_absolute_x(&mut self) {
        self.absolute_address_x_indexed();
        self.ora(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn ora_absolute_y(&mut self) {
        self.absolute_address_y_indexed();
        self.ora(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn ora_indirect_x(&mut self) {
        self.indirect_address_x_indexed();
        self.ora(self.read(self.address_bus));
        self.cycles = 6;
    }
    fn ora_indirect_y(&mut self) {
        self.indirect_address_y_indexed();
        self.ora(self.read(self.address_bus));
        self.cycles = 5;
    }

    fn and_immediate(&mut self) {
        let input = self.fetch_byte();
        self.and(input);
        self.cycles = 2;
    }
    fn and_zero_page(&mut self) {
        let input = self.fetch_byte() as u16;
        self.and(self.read(input));
        self.cycles = 3;
    }
    fn and_zero_page_x(&mut self) {
        self.zero_page_address_x_indexed();
        self.and(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn and_absolute(&mut self) {
        self.absolute_address();
        self.and(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn and_absolute_x(&mut self) {
        self.absolute_address_x_indexed();
        self.and(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn and_absolute_y(&mut self) {
        self.absolute_address_y_indexed();
        self.and(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn and_indirect_x(&mut self) {
        self.indirect_address_x_indexed();
        self.and(self.read(self.address_bus));
        self.cycles = 6;
    }
    fn and_indirect_y(&mut self) {
        self.indirect_address_y_indexed();
        self.and(self.read(self.address_bus));
        self.cycles = 5;
    }

    fn eor_immediate(&mut self) {
        let input = self.fetch_byte();
        self.eor(input);
        self.cycles = 2;
    }
    fn eor_zero_page(&mut self) {
        let input = self.fetch_byte() as u16;
        self.eor(self.read(input));
        self.cycles = 3;
    }
    fn eor_zero_page_x(&mut self) {
        self.zero_page_address_x_indexed();
        self.eor(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn eor_absolute(&mut self) {
        self.absolute_address();
        self.eor(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn eor_absolute_x(&mut self) {
        self.absolute_address_x_indexed();
        self.eor(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn eor_absolute_y(&mut self) {
        self.absolute_address_y_indexed();
        self.eor(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn eor_indirect_x(&mut self) {
        self.indirect_address_x_indexed();
        self.eor(self.read(self.address_bus));
        self.cycles = 6;
    }
    fn eor_indirect_y(&mut self) {
        self.indirect_address_y_indexed();
        self.eor(self.read(self.address_bus));
        self.cycles = 5;
    }

    fn adc_immediate(&mut self) {
        let input = self.fetch_byte();
        self.adc(input);
        self.cycles = 2;
    }
    fn adc_zero_page(&mut self) {
        let input = self.fetch_byte() as u16;
        self.adc(self.read(input));
        self.cycles = 3;
    }
    fn adc_zero_page_x(&mut self) {
        self.zero_page_address_x_indexed();
        self.adc(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn adc_absolute(&mut self) {
        self.absolute_address();
        self.adc(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn adc_absolute_x(&mut self) {
        self.absolute_address_x_indexed();
        self.adc(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn adc_absolute_y(&mut self) {
        self.absolute_address_y_indexed();
        self.adc(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn adc_indirect_x(&mut self) {
        self.indirect_address_x_indexed();
        self.adc(self.read(self.address_bus));
        self.cycles = 6;
    }
    fn adc_indirect_y(&mut self) {
        self.indirect_address_y_indexed();
        self.adc(self.read(self.address_bus));
        self.cycles = 5;
    }

    fn sbc_immediate(&mut self) {
        let input = self.fetch_byte();
        self.sbc(input);
        self.cycles = 2;
    }
    fn sbc_zero_page(&mut self) {
        let input = self.fetch_byte() as u16;
        self.sbc(self.read(input));
        self.cycles = 3;
    }
    fn sbc_zero_page_x(&mut self) {
        self.zero_page_address_x_indexed();
        self.sbc(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn sbc_absolute(&mut self) {
        self.absolute_address();
        self.sbc(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn sbc_absolute_x(&mut self) {
        self.absolute_address_x_indexed();
        self.sbc(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn sbc_absolute_y(&mut self) {
        self.absolute_address_y_indexed();
        self.sbc(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn sbc_indirect_x(&mut self) {
        self.indirect_address_x_indexed();
        self.sbc(self.read(self.address_bus));
        self.cycles = 6;
    }
    fn sbc_indirect_y(&mut self) {
        self.indirect_address_y_indexed();
        self.sbc(self.read(self.address_bus));
        self.cycles = 5;
    }

    fn cmp_immediate(&mut self) {
        let input = self.fetch_byte();
        self.cmp(input);
        self.cycles = 2;
    }
    fn cmp_zero_page(&mut self) {
        let input = self.fetch_byte() as u16;
        self.cmp(self.read(input));
        self.cycles = 3;
    }
    fn cmp_zero_page_x(&mut self) {
        self.zero_page_address_x_indexed();
        self.cmp(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn cmp_absolute(&mut self) {
        self.absolute_address();
        self.cmp(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn cmp_absolute_x(&mut self) {
        self.absolute_address_x_indexed();
        self.cmp(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn cmp_absolute_y(&mut self) {
        self.absolute_address_y_indexed();
        self.cmp(self.read(self.address_bus));
        self.cycles = 4;
    }
    fn cmp_indirect_x(&mut self) {
        self.indirect_address_x_indexed();
        self.cmp(self.read(self.address_bus));
        self.cycles = 6;
    }
    fn cmp_indirect_y(&mut self) {
        self.indirect_address_y_indexed();
        self.cmp(self.read(self.address_bus));
        self.cycles = 5;
    }

    fn cpx_immediate(&mut self) {
        let input = self.fetch_byte();
        self.cpx(input);
        self.cycles = 2;
    }
    fn cpx_zero_page(&mut self) {
        let input = self.fetch_byte() as u16;
        self.cpx(self.read(input));
        self.cycles = 3;
    }
    fn cpx_absolute(&mut self) {
        self.absolute_address();
        self.cpx(self.read(self.address_bus));
        self.cycles = 4;
    }

    fn cpy_immediate(&mut self) {
        let input = self.fetch_byte();
        self.cpy(input);
        self.cycles = 2;
    }
    fn cpy_zero_page(&mut self) {
        let input = self.fetch_byte() as u16;
        self.cpy(self.read(input));
        self.cycles = 3;
    }
    fn cpy_absolute(&mut self) {
        self.absolute_address();
        self.cpy(self.read(self.address_bus));
        self.cycles = 4;
    }

    fn brk(&mut self) {
        self.program_counter += 1;
        self.push((self.program_counter >> 8) as u8);
        self.push(self.program_counter as u8);
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
        let low: u8 = self.read(0xFFFE);
        let high: u8 = self.read(0xFFFF);
        self.program_counter = ((high as u16) << 8) | (low as u16);
        self.cycles = 7;
    }

    fn rti(&mut self) {
        let temp = self.pull();
        self.flag_carry = (temp & 0x01) != 0;
        self.flag_zero = (temp & 0x02) != 0;
        self.flag_interrupt_disable = (temp & 0x04) != 0;
        self.flag_decimal = (temp & 0x08) != 0;
        self.flag_overflow = (temp & 0x40) != 0;
        self.flag_negative = (temp & 0x80) != 0;

        let low = self.pull() as u16;
        let high = self.pull() as u16;

        self.program_counter = (high << 8) | low;

        self.cycles = 6;
    }

    fn bit_zero_page(&mut self) {
        let input = self.fetch_byte() as u16;
        self.bit(self.read(input));
        self.cycles = 3;
    }
    fn bit_absolute(&mut self) {
        self.absolute_address();
        self.bit(self.read(self.address_bus));
        self.cycles = 4;
    }
}
