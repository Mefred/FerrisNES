mod cpu;

use cpu::CPU;

fn main() {
    let mut cpu = CPU::new();
    cpu.load_rom("game.nes");
    cpu.reset();
}
