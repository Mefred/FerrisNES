mod cpu;

use cpu::CPU;

fn main() {
    let mut cpu = CPU::new();
    cpu.load_rom("2_ReadWrite.nes");
    cpu.reset();
    cpu.run();
    println!("RAM[0000] = {:02X}", cpu.read_debug(0x0000));
    println!("RAM[0001] = {:02X}", cpu.read_debug(0x0001));
    println!("RAM[0002] = {:02X}", cpu.read_debug(0x0002));
    println!("RAM[0550] = {:02X}", cpu.read_debug(0x0550));
}
