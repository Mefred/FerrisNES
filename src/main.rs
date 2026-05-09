mod cpu;

use cpu::CPU;

fn main() {
    let mut cpu = CPU::new();
    cpu.load_rom("roms/5_Instructions1.nes");
    cpu.reset();
    cpu.run();
    println!("{:02X}", cpu.read_debug(0x0000));
    println!("{:02X}", cpu.read_debug(0x0001));
    println!("{:02X}", cpu.read_debug(0x0002));
    println!("{:02X}", cpu.read_debug(0x0003));
    println!("{:02X}", cpu.read_debug(0x0004));
    println!("{:02X}", cpu.read_debug(0x0005));
    println!("{:02X}", cpu.read_debug(0x0006));
    println!("{:02X}", cpu.read_debug(0x0007));
    println!("{:02X}", cpu.read_debug(0x0008));
    println!("{:02X}", cpu.read_debug(0x0009));
    println!("{:02X}", cpu.read_debug(0x000A));
    println!("{:02X}", cpu.read_debug(0x000B));
    println!("{:02X}", cpu.read_debug(0x000C));
}
