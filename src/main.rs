use std::fs;
use std::io::{stdout, Write};
use armv6_m::structure::{MAX_RAM, MAX_ROM, MEMORY_MAX_ADDRESSABLE_ADDRESS, REGISTER_COUNT, ROM_PHYSICAL_ADDRESS};
use armv6_m::structure::Register::{PC};
use elfy::Elf;
use elfy::prelude::SectionData;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of file to flash in memory
    #[arg(short, long)]
    file: Option<String>,

    /// The input file is an ELF32
    #[arg(long, action)]
    elf: bool,

    /// Prints every action details, such as image loading
    #[arg(short, long, action)]
    verbose: bool,

    /// Checks every reserved space and avoids trying unsafe actions
    #[arg(short, long, action)]
    safe_mode: bool,
}

fn main() {
    let args = Args::parse();

    println!("==== START INIT ====");
    let mut memory = vec![0u8; MEMORY_MAX_ADDRESSABLE_ADDRESS as usize];

    let nb_written_bytes: usize;

    if let Some(file_path) = args.file {
        nb_written_bytes = init_with_file(&mut memory, file_path, args.verbose, args.elf);
    }
    else {
        // TODO implement instruction array playground
        nb_written_bytes = 0;
    }

    // If writen data quantity exceeds available memory
    if nb_written_bytes > MAX_ROM as usize {
        panic!("Max ROM size ({}o) exceeded!", MAX_ROM)
    }

    println!("Bytes written in ROM memory: {}", nb_written_bytes);

    if args.safe_mode {
        print!("Checking reserved spaces: ");
        // CODE
        print!("Code");
        stdout().flush().ok();
        for address in MAX_ROM - 1..0x10000000 {
            if memory[address as usize] != 0 {
                panic!("Data wrote in reserved Code space at address {:X}", address)
            }
        }

        // TODO FICR, UIRC (unknown size)

        // RAM
        print!(", RAM");
        stdout().flush().ok();
        for address in 0x20000000 + MAX_RAM - 1..0x40000000_u32 {
            if memory[address as usize] != 0 {
                panic!("Data wrote in reserved RAM space at address {:X}", address)
            }
        }

        // APB Peripherals
        print!(", APB Peripherals");
        stdout().flush().ok();
        for address in 0x40080000 - 1..0x50000000_u32 {
            if memory[address as usize] != 0 {
                panic!("Data wrote in reserved APB Peripherals space at address {:X}", address)
            }
        }

        // AHB Peripherals
        print!(", AHB Peripherals");
        stdout().flush().ok();
        for address in (0x50080000 - 1..0xE0000000_u32).rev() {
            if memory[address as usize] != 0 {
                panic!("Data wrote in reserved AHB Peripherals space at address {:X}", address)
            }
        }

        // Private Peripheral Bus
        println!(", Private Peripheral Bus");
        stdout().flush().ok();
        for address in (0xE0100000 - 1..0xFFFFFFFF_u32).rev() {
            if memory[address as usize] != 0 {
                panic!("Data wrote in reserved Private Peripheral Bus space at address {:X}", address)
            }
        }
    }

    println!();
    println!("==== END INIT ====");
    println!();

    if args.verbose {
        println!("==== START ROM ====");
        //for i in (0..nb_written_bytes).step_by(2) {
        for i in (0..20).step_by(2) {
            println!(
                "{:#04x}:\t{:02X}{:02X}\t{:#010b} {:#010b}",
                i,
                memory[i],
                memory[i + 1],
                memory[i],
                memory[i + 1]
            )
        }
        println!("==== END ROM ====");
    }
    println!();

    println!("==== START RUNTIME ====");

    // Empty all registers
    let mut registers: [u32; REGISTER_COUNT as usize] = [0; REGISTER_COUNT as usize];

    registers[PC.index()] = 0x0000;

    let mut running: bool = true;
    while running {

        let instruction = memory_read_u8(&memory, registers[PC.index()] as usize);
        registers[PC.index()] += 2;

        println!("{:b}", instruction);

        running = false;
    }

    println!("==== END RUNTIME ====");
}

/* ==== FUNCTIONS ==== */

/// Inits memory with given file path
fn init_with_file(memory: &mut [u8], file_path: String, verbose: bool, elf: bool) -> usize {
    println!("==== START READING FILE ====");
    println!("Path: {file_path}");

    let nb_written_bytes: usize;

    if elf {
        nb_written_bytes = read_elf_file(memory, file_path, verbose);
    }
    else {
        nb_written_bytes = read_hex_file(memory, file_path, verbose);
    }

    println!("==== END READING FILE ====");
    println!();

    nb_written_bytes
}

/// <https://gitlab.com/qemu-project/qemu/-/blob/master/hw/core/loader.c#L1883  >
/// Reads an Hex file and returns the number of bytes written into the memory
fn read_hex_file(memory: &mut [u8], file_path: String, verbose: bool) -> usize {
    let file = fs::read_to_string(&file_path).unwrap();

    let mut bytes_written: usize = 0;
    let mut extended_linear_address: u32 = 0x0000000000000000;
    let mut in_v1_block: bool = false;

    for (line_index, mut line) in file.split('\n').enumerate() {
        if line.starts_with(':') {
            line = line.split_at(1).1;
            let data = hex::decode(line).unwrap();

            let nb_data: usize = data[0] as usize;
            let address = u16::from_be_bytes([data[1], data[2]]);
            let record_type = data[3];

            if verbose && in_v1_block {
                println!("{}:\tnb: {},\taddress: {},\ttype: {:#04X}", line_index, nb_data, address, record_type);
            }

            match record_type {
                // Block start
                0x0A => {
                    if nb_data == 4 {
                        let bytes_as_u16 = u16::from_be_bytes([data[4], data[5]]);

                        // Microbit V1 Block
                        if bytes_as_u16 == 0x9900 {
                            in_v1_block = true;

                            if verbose {
                                println!("Block start - microbit V1");
                            }
                        }
                        // Microbit V2 Block = 0x9900
                        if bytes_as_u16 == 0x9903 {

                            if verbose {
                                println!("Block start - microbit V2");
                            }
                        }
                    }
                },
                // Block end
                0x0C | 0x0B => {
                    if nb_data == 0 {
                        in_v1_block = false;

                        if verbose {
                            println!("Block end microbit");
                        }
                    }
                },
                // Extended linear address
                0x04 => {
                    if nb_data == 2 {
                        extended_linear_address = u32::from_be_bytes([data[4], data[5], 0, 0]);
                    }
                    else {
                        panic!("Malformed data size at line {line_index}")
                    }
                },
                // Data
                0x00 => {
                    if in_v1_block {
                        for byte_index in 0..nb_data {
                            memory[extended_linear_address as usize + address as usize + byte_index] = data[byte_index + 4];
                        }
                        bytes_written += nb_data
                    }
                },
                _ => {}
            }
        } else if let Some(character) = line.chars().next() {
            panic!("Unexpected special char {character} at line {line_index}");
        }
    }

    bytes_written
}


/// Reads an elf32 ARM file and returns the number of bytes written into the memory
fn read_elf_file(memory: &mut [u8], file_path: String, verbose: bool) -> usize {
    let elf = Elf::load(&file_path).expect("Cannot open image");

    let mut nb_written_bytes: usize = 0;

    let origin = elf.header().entry();
    println!("Origin: {origin}");

    let text_section = elf.try_get_section(".text").expect("The section doesn't exist!");

    if let SectionData::Bytes(bytes) = text_section.data() {
        println!("Section size: {}", bytes.len());

        if verbose {
            for (index, byte) in bytes.iter().enumerate() {
                nb_written_bytes += 1;
                println!("{:x}: {:#010b}", index, byte);
                memory[index] = *byte;
            }
        }
        else {
            for (index, byte) in bytes.iter().enumerate() {
                nb_written_bytes += 1;
                memory[index] = *byte;
            }
        }
    }
    else {
        panic!("No data found in .text section")
    }

    nb_written_bytes + 1
}

/// Reads one byte from memory
fn memory_read_u8(memory: &[u8], address: usize) -> u8 {
    memory[address]
}

/// Reads four byte from memory
fn memory_read_4u8(memory: &[u8], address: usize) -> [u8;4] {
    [
        memory[address],
        memory[address + 1],
        memory[address + 2],
        memory[address + 3]
    ]
}

/// Reads one byte from memory, returned as u16
fn memory_read_u16(memory: &[u8], address: usize) -> u16 {
    let byte_array = [
        memory[address],
        memory[address + 1]
    ];
    u16::from_be_bytes(byte_array)
}

/// Reads one byte from memory, returned as u32
fn memory_read_u32(memory: &[u8], address: usize) -> u32 {
    let byte_array = [
        memory[address],
        memory[address + 1],
        memory[address + 2],
        memory[address + 3]
    ];
    u32::from_be_bytes(byte_array)
}

fn swap32(num: u32) -> u32 {
    ((num>>24)  & 0xff)     | // move byte 3 to byte 0
    ((num<<8)   & 0xff0000) | // move byte 1 to byte 2
    ((num>>8)   & 0xff00)   | // move byte 2 to byte 1
    ((num<<24)  & 0xff000000)
}

fn as_u32_be(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) << 24) +
    ((array[1] as u32) << 16) +
    ((array[2] as u32) <<  8) +
    (array[3] as u32)
}

fn as_u32_le(array: &[u8; 4]) -> u32 {
    (array[0] as u32) +
    ((array[1] as u32) <<  8) +
    ((array[2] as u32) << 16) +
    ((array[3] as u32) << 24)
}