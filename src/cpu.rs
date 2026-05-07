use std::fs;

pub struct CPU {
    program_counter: u16,
    register_a: u8,
    register_x: u8,
    register_y: u8,
    ram: [u8; 0x800],
    rom: [u8; 0x8000],
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
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        let rom_file = fs::read(path).unwrap();

        self.rom.copy_from_slice(&rom_file[0x10..0x10 + 0x8000]);
    }

    fn read(&self, address: u16) -> u8 {
        if address < 0x800 {
            return self.ram[address as usize];
        }
        if address >= 0x8000 {
            return self.rom[(address - 0x8000) as usize];
        }
        panic!("Unhandled address");
    }

    pub fn reset(&mut self) {
        let pcl = self.read(0xFFFC);
        let pch = self.read(0xFFFD);

        self.program_counter = ((pch as u16) << 8) | (pcl as u16)
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.read(self.program_counter);
            self.program_counter += 1;

            match opcode {
                0x02 => break, // HTL
                0xA0 => {
                    self.register_y = self.read(self.program_counter);
                    self.program_counter += 1;
                } // LDY Immediate
                0xA2 => {
                    self.register_x = self.read(self.program_counter);
                    self.program_counter += 1;
                } // LDX Immediate
                0xA9 => {
                    self.register_a = self.read(self.program_counter);
                    self.program_counter += 1;
                } // LDA Immediate

                _ => todo!(),
            }
        }
    }
}
