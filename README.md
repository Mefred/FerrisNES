# FerrisNES

A Nintendo Entertainment System (NES) emulator written in Rust.

---

## Features

### CPU
- 6502 CPU emulation
- Opcode decoding system
- Status flag handling
- Addressing modes
- Cycle tracking

### Emulator Core
- ROM loading
- Memory mapping
- Modular architecture
- Written fully in Rust

---

## Goals

- Run commercial NES games
- Accurate CPU timing
- Full PPU implementation
- Mapper support
- Controller input
- Audio (APU) support
- Good performance and clean code structure

---

## Current Progress

FerrisNES is currently in active development.

Implemented so far:
- CPU core
- Large portion of instructions
- Basic memory system
- ROM loading

Planned next:
- More accurate timing
- Complete PPU rendering
- Mapper support
- Game compatibility improvements

---

## Screenshots

> Screenshots coming soon.

---

## Building

### Requirements
- Rust
- Cargo

### Clone the repository

```bash
git clone https://github.com/Mefred/FerrisNES.git
cd FerrisNES
