use std::{env};
use armv6_m::{MEMORY_MAX_ADDRESSABLE_ADDRESS, R_COUNT};
use armv6_m::ConditionFlag::PL;
use armv6_m::Register::{APSR, PC};
use elfy::Elf;
use elfy::prelude::SectionData;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut memory = vec![0u32; MEMORY_MAX_ADDRESSABLE_ADDRESS];

    if args.len() > 1 {
        let file_path = args.get(1).unwrap();
        let elf = Elf::load(file_path).expect("Cannot open image");

        let origin = elf.header().entry();
        println!("origin: {origin}");

        let text_section = elf.try_get_section(".init").expect("The section doesn't exist!");

        if let SectionData::Bytes(bytes) = text_section.data() {
            for i in (0..bytes.len()).step_by(4) {
                println!("{:#010b} {:#010b} {:#010b} {:#010b}", bytes[i+3], bytes[i+2], bytes[i+1], bytes[i])
            }
        }

        /*
        let mut origin = file.read_u32::<LittleEndian>().unwrap();
        origin = swap32(origin);

        let max_read = MEMORY_MAX_ADDRESSABLE_ADDRESS as u32 - origin;
        println!("{} - {} = {}", MEMORY_MAX_ADDRESSABLE_ADDRESS, origin, max_read);

        */
        //println!("memory pointer: {}", &p);
        //memory[p as usize] = 0b101;
        //println!("data at pointer: {}", &memory[p as usize]);
        println!();
    }

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

fn memory_read(memory: &[u32], address: u32) -> u32 {
    memory[address as usize]
}

fn swap32(num: u32) -> u32 {
    ((num>>24)  & 0xff)     | // move byte 3 to byte 0
    ((num<<8)   & 0xff0000) | // move byte 1 to byte 2
    ((num>>8)   & 0xff00)   | // move byte 2 to byte 1
    ((num<<24)  & 0xff000000)
}