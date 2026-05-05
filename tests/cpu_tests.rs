use FerrisNES::CPU;

//
// =======================================================
// 0xA9 - LDA Immediate
// =======================================================
//

#[test]
fn lda_immediate_loads_value() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xa9, 0x10, 0x00]);
    assert_eq!(cpu.register_a, 0x10);
}

#[test]
fn lda_sets_zero_flag() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xa9, 0x00, 0x00]);
    assert_ne!(cpu.status & 0b0000_0010, 0);
}

#[test]
fn lda_sets_negative_flag() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xa9, 0x80, 0x00]);
    assert_ne!(cpu.status & 0b1000_0000, 0);
}

//
// =======================================================
// 0xAA - TAX
// =======================================================
//

#[test]
fn tax_copies_a_to_x() {
    let mut cpu = CPU::new();
    cpu.register_a = 0x22;

    cpu.interpret(vec![0xaa, 0x00]);

    assert_eq!(cpu.register_x, 0x22);
}

#[test]
fn tax_sets_zero_flag_when_x_is_zero() {
    let mut cpu = CPU::new();
    cpu.register_a = 0x00;

    cpu.interpret(vec![0xaa, 0x00]);

    assert_ne!(cpu.status & 0b0000_0010, 0);
}

//
// =======================================================
// 0xE8 - INX
// =======================================================
//

#[test]
fn inx_increments_x() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xe8, 0x00]);

    assert_eq!(cpu.register_x, 1);
}

#[test]
fn inx_wraps_overflow() {
    let mut cpu = CPU::new();
    cpu.register_x = 0xff;

    cpu.interpret(vec![0xe8, 0x00]);

    assert_eq!(cpu.register_x, 0x00);
}

//
// =======================================================
// 0xCA - DEX (NEW)
// =======================================================
//

#[test]
fn dex_decrements_x() {
    let mut cpu = CPU::new();
    cpu.register_x = 5;

    cpu.interpret(vec![0xca, 0x00]);

    assert_eq!(cpu.register_x, 4);
}

#[test]
fn dex_wraps_underflow() {
    let mut cpu = CPU::new();
    cpu.register_x = 0x00;

    cpu.interpret(vec![0xca, 0x00]);

    assert_eq!(cpu.register_x, 0xff);
}

//
// =======================================================
// 0xA2 - LDX Immediate (NEW)
// =======================================================
//

#[test]
fn ldx_loads_value() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xa2, 0x33, 0x00]);

    assert_eq!(cpu.register_x, 0x33);
}

#[test]
fn ldx_sets_negative_flag() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xa2, 0x80, 0x00]);

    assert_ne!(cpu.status & 0b1000_0000, 0);
}

//
// =======================================================
// 0x69 - ADC Immediate (NEW)
// =======================================================
//

#[test]
fn adc_adds_values() {
    let mut cpu = CPU::new();

    cpu.interpret(vec![0xa9, 0x10, 0x69, 0x05, 0x00]);

    assert_eq!(cpu.register_a, 0x15);
}

#[test]
fn adc_overflow_wraps() {
    let mut cpu = CPU::new();

    cpu.interpret(vec![0xa9, 0xff, 0x69, 0x02, 0x00]);

    assert_eq!(cpu.register_a, 0x01);
}

//
// =======================================================
// 0xC9 - CMP Immediate (NEW)
// =======================================================
//

#[test]
fn cmp_sets_zero_when_equal() {
    let mut cpu = CPU::new();

    cpu.interpret(vec![0xa9, 0x10, 0xc9, 0x10, 0x00]);

    assert_ne!(cpu.status & 0b0000_0010, 0);
}

#[test]
fn cmp_sets_negative_when_less() {
    let mut cpu = CPU::new();

    cpu.interpret(vec![0xa9, 0x10, 0xc9, 0x20, 0x00]);

    assert_ne!(cpu.status & 0b1000_0000, 0);
}

//
// =======================================================
// 0x4C - JMP (NEW)
// =======================================================
//

#[test]
fn jmp_changes_execution_flow() {
    let mut cpu = CPU::new();

    cpu.interpret(vec![0x4c, 0x06, 0x00, 0xa9, 0x11, 0xa9, 0x22, 0x00]);

    assert_eq!(cpu.register_a, 0x22);
}

//
// =======================================================
// 0xF0 - BEQ (NEW)
// =======================================================
//

#[test]
fn beq_takes_branch_when_zero_flag_set() {
    let mut cpu = CPU::new();
    cpu.status |= 0b0000_0010;

    cpu.interpret(vec![0xf0, 0x02, 0xa9, 0x99, 0x00]);

    assert_eq!(cpu.register_a, 0x00);
}

#[test]
fn beq_does_not_branch_when_zero_flag_clear() {
    let mut cpu = CPU::new();
    cpu.status &= !0b0000_0010;

    cpu.interpret(vec![0xf0, 0x02, 0xa9, 0x77, 0x00]);

    assert_eq!(cpu.register_a, 0x77);
}

//
// =======================================================
// 0x8D - STA Absolute (NEW EXPECTED BEHAVIOR)
// =======================================================
//

#[test]
fn sta_writes_a_to_memory() {
    let mut cpu = CPU::new();

    cpu.register_a = 0x42;

    cpu.interpret(vec![0x8d, 0x10, 0x00, 0x00]);

    // expected: memory[0x0010] == 0x42
    // (you may expose memory in CPU struct or test indirectly)
}

//
// =======================================================
// 0xAD - LDA Absolute (NEW EXPECTED BEHAVIOR)
// =======================================================
//

#[test]
fn lda_absolute_reads_memory() {
    let mut cpu = CPU::new();

    cpu.interpret(vec![0xad, 0x10, 0x00, 0x00]);

    // expected: register_a == memory[0x0010]
}

//
// =======================================================
// SYSTEM BEHAVIOR
// =======================================================
//

#[test]
fn brk_stops_execution() {
    let mut cpu = CPU::new();

    cpu.interpret(vec![0xa9, 0x01, 0x00, 0xa9, 0x99]);

    assert_eq!(cpu.register_a, 0x01);
}

use std::panic::{AssertUnwindSafe, catch_unwind};

#[test]
fn unknown_opcode_should_fail_safely() {
    let mut cpu = CPU::new();

    let result = catch_unwind(AssertUnwindSafe(|| {
        cpu.interpret(vec![0xff, 0x00]);
    }));

    assert!(result.is_err());
}
