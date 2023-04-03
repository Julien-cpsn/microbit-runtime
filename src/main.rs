use std::{env};
use armv6_m::{MEMORY_MAX_ADDRESSABLE_ADDRESS, R_COUNT};
use armv6_m::ConditionFlag::PL;
use armv6_m::Register::{APSR, PC};

fn main() {
    let args: Vec<String> = env::args().collect();

    let memory = vec![0u8; MEMORY_MAX_ADDRESSABLE_ADDRESS];

    // Empty all registers
    let mut reg: [u32; R_COUNT] = [0; R_COUNT];

    // Set last used condition flag as PL aka zero
    reg[APSR.index()] = PL as u32;

    // Program counter start
    const PC_START: i32 = 0x3000;

    reg[PC.index()] = PC_START as u32;

    let mut running: bool = true;
    while running {
        reg[PC.index()] = reg[PC.index()] + 1;
        let instruction = memory_read(&memory, reg[PC.index()]);
        let operation = instruction as u32 >> 12;

        println!("op {}", operation as u8);

        running = false;
    }
}

fn memory_read(memory: &[u8], address: u32) -> u8 {
    memory[address as usize]
}