use std::fs;

type Instr = fn(&mut CPU);

pub struct CPU {
    pub program_counter: u16,
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,

    ram: Vec<u8>,
    rom: [u8; 0x8000],

    instructions: [Instr; 256],
}

impl CPU {
    pub fn new() -> Self {
        let mut cpu = Self {
            program_counter: 0,
            register_a: 0,
            register_x: 0,
            register_y: 0,
            ram: vec![0; 0x800],
            rom: [0; 0x8000],
            instructions: [CPU::unimplemented; 256],
        };

        cpu.bind_instruction();
        cpu
    }

    fn unimplemented(_cpu: &mut CPU) {
        panic!("unimplemented opcode");
    }

    // -------------------------
    // Instruction table setup
    // -------------------------

    fn bind_instruction(&mut self) {
        self.instructions[0xA9] = Self::lda_immediate;
        self.instructions[0xA5] = Self::lda_zero_page;
        self.instructions[0xAD] = Self::lda_absolute;

        self.instructions[0xA2] = Self::ldx_immediate;
        self.instructions[0xA0] = Self::ldy_immediate;

        self.instructions[0x85] = Self::sta_zero_page;
        self.instructions[0x8D] = Self::sta_absolute;
    }

    // -------------------------
    // ROM loading
    // -------------------------

    pub fn load_rom(&mut self, path: &str) {
        let rom_file = fs::read(path).unwrap();

        self.rom.copy_from_slice(&rom_file[0x10..0x10 + 0x8000]);
    }

    // -------------------------
    // Memory system
    // -------------------------

    fn read(&self, address: u16) -> u8 {
        if address < 0x800 {
            return self.ram[address as usize];
        }
        if address >= 0x8000 {
            return self.rom[(address - 0x8000) as usize];
        }
        panic!("Unhandled address");
    }

    fn write(&mut self, address: u16, data: u8) {
        if address < 0x800 {
            self.ram[address as usize] = data;
        }
    }

    // -------------------------
    // Helpers
    // -------------------------

    fn fetch_byte(&mut self) -> u8 {
        let byte = self.read(self.program_counter);
        self.program_counter += 1;
        byte
    }

    // -------------------------
    // CPU control
    // -------------------------

    pub fn reset(&mut self) {
        let pcl = self.read(0xFFFC);
        let pch = self.read(0xFFFD);

        self.program_counter = ((pch as u16) << 8) | (pcl as u16)
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.fetch_byte();
            (self.instructions[opcode as usize])(self);
        }
    }

    // -------------------------
    // Instructions
    // -------------------------

    fn lda_immediate(cpu: &mut CPU) {
        cpu.register_a = cpu.fetch_byte();
    }

    fn lda_zero_page(cpu: &mut CPU) {
        let addr = cpu.fetch_byte() as u16;
        cpu.register_a = cpu.read(addr);
    }

    fn lda_absolute(cpu: &mut CPU) {
        let low = cpu.fetch_byte() as u16;
        let high = cpu.fetch_byte() as u16;

        let addr = (high << 8) | low;

        cpu.register_a = cpu.read(addr);
    }

    fn ldx_immediate(cpu: &mut CPU) {
        cpu.register_x = cpu.fetch_byte();
    }

    fn ldy_immediate(cpu: &mut CPU) {
        cpu.register_y = cpu.fetch_byte();
    }

    fn sta_zero_page(cpu: &mut CPU) {
        let addr = cpu.fetch_byte() as u16;
        cpu.write(addr, cpu.register_a);
    }

    fn sta_absolute(cpu: &mut CPU) {
        let address_low = cpu.fetch_byte() as u16;
        let address_high = cpu.fetch_byte() as u16;
        cpu.write(address_high * 256 + address_low, cpu.register_a);
    }
}
