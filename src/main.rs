use std::fs;
use std::fs::File;
use std::io::Read;
use armv6_m::structure::{MAX_ROM, MEMORY_MAX_ADDRESSABLE_ADDRESS, REGISTER_COUNT, ROM_PHYSICAL_ADDRESS};
use armv6_m::structure::ConditionFlag::PL;
use armv6_m::structure::Register::{APSR, PC};
use elfy::Elf;
use elfy::prelude::SectionData;
use clap::Parser;
use ihex::{Reader, ReaderOptions};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of binary file to flash in memory
    #[arg(short, long)]
    file: Option<String>,

    #[arg(long, action)]
    elf: bool,

    #[arg(short, long, action)]
    verbose: bool,
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
        // TODO
        nb_written_bytes = 0;
    }

    // If writen data quantity exceeds available memory
    if nb_written_bytes > MAX_ROM as usize {
        panic!("Max ROM size ({}o) exceeded!", MAX_ROM)
    }

    println!("Bytes written in ROM memory: {}", nb_written_bytes);
    println!();
    println!("==== END INIT ====");

    if args.verbose {
        println!("==== START ROM ====");
        for i in (0..20).step_by(2) {
            println!(
                "{:#04x}: {:#010b} {:#010b}",
                i,
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

    let mut running: bool = true;
    while running {

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

/// Reads an Hex file and returns the number of bytes written into the memory
fn read_hex_file(memory: &mut [u8], file_path: String, verbose: bool) -> usize {
    let file = fs::read_to_string(&file_path).unwrap();

    let mut bytes_written: usize = 0;
    let mut in_v1_block: bool = false;

    if verbose {
        for (line_index, mut line) in file.split('\n').enumerate() {
            if line.starts_with(':') {
                line = line.split_at(1).1;
                let data = hex::decode(line).unwrap();

                let nb_data: usize = data[0] as usize;
                let address = u16::from_be_bytes([data[1], data[2]]);
                let record_type = data[3];

                println!("{}: nb: {}, address: {}, type: {:#04X}", line_index, nb_data, address, record_type);

                // Block start
                match record_type {
                    0x0A => {
                        if nb_data == 4 {
                            let bytes_as_u16 = u16::from_be_bytes([data[4], data[5]]);

                            // Microbit V1 Block
                            if bytes_as_u16 == 0x9900 {
                                in_v1_block = true;
                                println!("Block start - microbit V1");
                            }
                            // Microbit V2 Block = 0x9900
                            if bytes_as_u16 == 0x9903 {
                                println!("Block start - microbit V2")
                            }
                        }
                    },
                    0x0C | 0x0B => {
                        if nb_data == 0 {
                            in_v1_block = false;
                            println!("Block end microbit")
                        }
                    },
                    0x00 => {
                        if in_v1_block {
                            for byte_index in 0..nb_data {
                                memory[address as usize + byte_index] = data[byte_index + 4];
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
    }
    else {
        for (line_index, line) in file.split('\n').enumerate() {
            if line.starts_with(':') {
                let data = hex::decode(line.split_at(1).1).unwrap();

                let nb_data: usize = data[0] as usize;
                let address = u16::from_be_bytes([data[1], data[2]]);
                let record_type = data[3];

                // Block start
                match record_type {
                    0x0A => {
                        if nb_data == 4 {
                            let bytes_as_u16 = u16::from_be_bytes([data[4], data[5]]);

                            // Microbit V1 Block
                            if bytes_as_u16 == 0x9900 {
                                in_v1_block = true;
                            }
                            // Microbit V2 Block = 0x9900
                        }
                    },
                    0x0C | 0x0B => {
                        if nb_data == 0 {
                            in_v1_block = false;
                        }
                    },
                    0x00 => {
                        if in_v1_block {
                            for byte_index in 0..nb_data {
                                memory[address as usize + byte_index] = data[byte_index + 4];
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

/// Reads one byte from memory, returned as u32
fn memory_read_u32(memory: &[u8], address: usize) -> u32 {
    let byte_array = [
        memory[address],
        memory[address + 1],
        memory[address + 2],
        memory[address + 3]
    ];
    as_u32_be(&byte_array)
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