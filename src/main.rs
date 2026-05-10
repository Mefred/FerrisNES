mod cpu;

use cpu::CPU;

fn main() {
    let mut cpu = CPU::new();
    cpu.load_rom("roms/6_Instructions2.nes");
    cpu.reset();
    cpu.run();
}
