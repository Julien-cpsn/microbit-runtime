use armv6_m::{R_COUNT, Register};
use armv6_m::ConditionFlag::PL;

fn main() {
    Register::R0(1);

    let mut reg: &[u32; R_COUNT] = &[0; R_COUNT];

    /*
    reg[Register::APSR] = PL;

    const PC_START: i32 = 0x3000;

    reg[PC] = PC_START;

    let running: bool = true;
    while running {
        reg[PC] = reg[PC] + 1;
        //let instruction = read(reg[PC]);
    }*/
}
