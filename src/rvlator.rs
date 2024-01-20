#![allow(dead_code)]
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::io::Read;
//use std::println as debug;

/// bitmask32(width, position)
macro_rules! bitmask32 {
    ($width:expr, $pos:expr) => {{
        (((1 as u32) << $width) - 1) << $pos
    }};
}

/// printinst(instruction)
#[allow(unused_macros)]
macro_rules! printinst {
    ($inst: expr) => {{
        println!("0x{:08x}", $inst);
    }};
}

/// getfield32(value_32bit, width, position)
macro_rules! getfield32 {
    ($val:expr, $width:expr, $pos:expr) => {{
        ($val & bitmask32!($width, $pos)) >> $pos
    }};
}

/// Check for the valid register
macro_rules! sanitizereg {
    ($reg:expr) => {{
        assert!($reg < 32);
    }};
}

//LATER: Check for the corner cases which may break it
#[inline]
fn signext12to64(val:u32) -> u64 {
    if (val >> (12 - 1)) == 0x1 {
        val as u64 | (u64::MAX - ((1 << 12) - 1))
    }
    else {
        val as u64
    }
}

//LATER: Check for the corner cases which may break it
#[inline]
fn signext20to64(val:u32) -> u64 {
    if (val >> (20 - 1)) == 0x1 {
        val as u64 | (u64::MAX - ((1 << 20) - 1))
    }
    else {
        val as u64
    }
}

#[inline]
fn signext_nto64(val:u64, bits: u64) -> u64 {
    if (val >> (bits - 1)) == 0x1 {
        val | (u64::MAX - ((1 << bits) - 1))
    }
    else {
        val
    }
}

// Color Codes for terminal
const COLOR_RESET:&str = "\x1b[0m";
const COLOR_GREY:&str = "\x1b[1;30m";
const COLOR_RED:&str = "\x1b[1;31m";
const COLOR_GREEN:&str = "\x1b[1;32m";
const COLOR_BROWN:&str = "\x1b[1;33m";
const COLOR_BLUE:&str = "\x1b[1;34m";
const COLOR_PINK:&str = "\x1b[1;35m";
const COLOR_AQUA:&str = "\x1b[1;36m";

const RESET_VECTOR: u64 = 0x0;
const ISIZE: u8 = 32;
const IALIGN: u8 = 32;
const XLEN: u8 = 64;
const HALFWORD: u8 = 16;
const WORD: u8 = 32;
const DOUBLEWORD: u8 = 64;
const QUADWORD: u8 = 128;

// Register Encoding. Used also for
// indexing into the name array REGNAMES
const REG_ZERO:usize = 0;
const REG_RA :usize = 1 ;
const REG_SP :usize = 2;
const REG_GP :usize = 3;
const REG_TP :usize = 4;
const REG_T0 :usize = 5;
const REG_T1 :usize = 6;
const REG_T2 :usize = 7;
const REG_S0 :usize = 8;
const REG_S1 :usize = 9;
const REG_A0 :usize = 10;
const REG_A1 :usize = 11;
const REG_A2 :usize = 12;
const REG_A3 :usize = 13;
const REG_A4 :usize = 14;
const REG_A5 :usize = 15;
const REG_A6 :usize = 16;
const REG_A7 :usize = 17;
const REG_S2 :usize = 18;
const REG_S3 :usize = 19;
const REG_S4 :usize = 20;
const REG_S5 :usize = 21;
const REG_S6 :usize = 22;
const REG_S7 :usize = 23;
const REG_S8 :usize = 24;
const REG_S9 :usize = 25;
const REG_S10 :usize = 26;
const REG_S11 :usize = 27;
const REG_T3 :usize = 28;
const REG_T4 :usize = 29;
const REG_T5 :usize = 30;
const REG_T6 :usize = 31;

const INST_OPCODE_POS: u8 = 0;
const INST_OPCODE_WID: u8 = 7;
const INST_RD_POS: u8 = 7;
const INST_RD_WID: u8 = 5;
const INST_FUNCT3_POS: u8 = 12;
const INST_FUNCT3_WID: u8 = 3;
const INST_RS1_POS: u8 = 15;
const INST_RS1_WID: u8 = 5;
const INST_RS2_POS: u8 = 20;
const INST_RS2_WID: u8 = 5;
const INST_FUNCT7_POS: u8 = 25;
const INST_FUNCT7_WID: u8 = 7;
const INST_SHAMT_POS:u8 = 20;
const INST_SHAMT_WID:u8 = 6;
const INST_IMM4_0_POS: u8 = INST_RD_POS;
const INST_IMM4_0_WID: u8 = INST_RD_WID;
const INST_IMM11_0_POS: u8 = INST_RS2_POS;
const INST_IMM11_0_WID: u8 = INST_RS2_WID + INST_FUNCT7_WID;
const INST_IMM11_5_POS: u8 = INST_FUNCT7_POS;
const INST_IMM11_5_WID: u8 = INST_FUNCT7_WID;
const INST_IMM31_12_POS: u8 = INST_FUNCT3_POS;
const INST_IMM31_12_WID: u8 = INST_FUNCT3_WID + INST_RS1_WID + INST_IMM11_0_WID;

const REGNAME: [&str; 32] = [
    "z0", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1",
    "a2", "a3", "a4", "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7",
    "s8", "s9", "sA", "sB", "t3", "t4", "t5", "t6",
];

enum RiscvException {
    InstructionAddressMisaligned,
    InstructionAccessFault,
    IllegalInstruction,
    Breakpoint,
    LoadAddressMisaligned,
    LoadAccessFault,
    StoreAmoAddressMisaligned,
    StoreAmoAccessFault,
    EcallUmode,
    EcallSmode,
    EcallMmode,
    InstructionPageFault,
    LoadPageFault,
    StoreAmoPageFault,
}

enum RiscvMemType {
    Vacant,
    MainMemory,
    IoMemory,
}

enum RiscvInstType {
    Bit16,
    Bit32,
    Illegal,
}

#[derive(Debug, PartialEq)]
enum RiscvCpuError {
    FetchError,
    DecodeError,
    ExecuteError,
}

struct RiscvCpu {
    // 64-bit 32 registers integer register unit
    ixu: [u64; 32],
    // program counter
    pc: u64,
    // Byte addressable memory
    mem: Vec<u8>,
}

impl RiscvCpu {
    // LATER: Singleton pattern to allow only one Cpu instance
    fn new(code: Vec<u8>) -> RiscvCpu {
        RiscvCpu {
            ixu: [0; 32],
            pc: RESET_VECTOR,
            mem: code.clone(),
        }
    }

    fn fetch(&self) -> Result<u32, RiscvCpuError> {
        if self.pc < self.mem.len().try_into().unwrap() {
            let idx = self.pc as usize; // LATER: Using `as` is lossy conversion
                                        // Instructions are stored in memory in 16-bit parcels which
                                        // follow little-endian order. ILEN encoding on the LSB side.
                                        // Fetching 32-bit instruction
            let inst = self.mem[idx] as u32
                | (self.mem[idx + 1] as u32) << 8
                | (self.mem[idx + 2] as u32) << 16
                | (self.mem[idx + 3] as u32) << 24;
            Ok(inst)
        } else {
            Err(RiscvCpuError::FetchError)
        }
    }
    
    fn execute(&mut self, inst: u32) -> Result<(), RiscvCpuError> {
        //32-bit Valid Instruction => xxxxxxxxxbbb11 (bbb != 111)
        //inst[1:0] field
        let enc: u32 = getfield32!(inst, 2, 0);
        //inst[4:2](bbb) field
        let bbb: u32 = getfield32!(inst, 3, 2);

        //Check if valid 32-bit instruction
        if enc != 0x3 || bbb == 0x7 {
            println!(
                "Error: Inval Inst: 0x{:08x}, enc: 0b{:02b}, bbb: 0b{:03b}",
                inst, enc, bbb
            );
            //Decode error when instruction is illegal which
            //are not allowed by RISC-V ISA. Illegal instructions
            //like inst[15:0] == 0 and inst[ILEN-1:0] == 1 do not
            //generate DecodeError even though they are ISA allowed
            //illegal instructions
            //LATER: Generate RiscvException::IllegalInstruction
            return Err(RiscvCpuError::DecodeError);
        }

        let opcode: u32 = getfield32!(inst, INST_OPCODE_WID, INST_OPCODE_POS);
        match opcode {
            0b0010111 => {
                let rd:usize = getfield32!(inst, INST_RD_WID, INST_RD_POS).try_into().unwrap();
                sanitizereg!(rd);
                let imm20:u32 = getfield32!(inst, INST_IMM31_12_WID, INST_IMM31_12_POS).try_into().unwrap();
                let simm20:u64 = signext20to64(imm20);
                println!("auipc {},{}", REGNAME[rd], simm20 as i64);
                self.ixu[rd] = self.pc + (simm20 << 12);
            }
            // Base ISA
            0b0110111 => { // lui
                let rd:usize = getfield32!(inst, INST_RD_WID, INST_RD_POS).try_into().unwrap();
                sanitizereg!(rd);
                let imm20:u32 = getfield32!(inst, INST_IMM31_12_WID, INST_IMM31_12_POS).try_into().unwrap();
                let simm20:u64 = signext20to64(imm20);
                println!("lui {},{}", REGNAME[rd], simm20 as i64);
                self.ixu[rd] = simm20 << 12;
            }
            // Base ISA
            0b0010011 => { // addi, slti, sltiu, andi, ori, xori, slli, srli, srai
                //Integer Register Immediate Instructions
                // Both rd and rs are usize instead of u32 to index into the ixu array
                let rd: usize = getfield32!(inst, INST_RD_WID, INST_RD_POS).try_into().unwrap();
                sanitizereg!(rd);
                let rs1: usize = getfield32!(inst, INST_RS1_WID, INST_RS1_POS).try_into().unwrap();
                sanitizereg!(rs1);
        
                let imm12:u32 = getfield32!(inst, INST_IMM11_0_WID, INST_IMM11_0_POS);
                let simm12:u64 = signext12to64(imm12);
                let funct3:u32 = getfield32!(inst, INST_FUNCT3_WID, INST_FUNCT3_POS);

                match funct3 {
                    0b000 => { //ADDI: x[rd] = x[rs1] + sext(immediate)
                        println!("addi {},{},{}", REGNAME[rd], REGNAME[rs1], simm12 as i64);
                        // Why wrapping_add? 0xfffffffffffffffc + 0xffffffffffffffff = 1fffffffffffffffb
                        // We need to discard 1 since this instruction ignores the Arithmetic Overflows
                        self.ixu[rd] = self.ixu[rs1].wrapping_add(simm12);
                    }
                    0b001 => { //SLLI: x[rd] = x[rs1] << shamt
                        // 0 <= shamt <= 63, imm12[5:0] or inst[25:20] are used as shift value
                        let shamt = getfield32!(inst, INST_SHAMT_WID, INST_SHAMT_POS);
                        println!("slli {},{},{}", REGNAME[rd], REGNAME[rs1], shamt);
                        self.ixu[rd] = self.ixu[rs1] << shamt;
                    }
                    0b010 => { //SLTI: x[rd] = 1 if x[rs1] <s sext(immediate) else x[rd] = 0
                        println!("slti {},{},{}", REGNAME[rd], REGNAME[rs1], simm12 as i64);
                        if (self.ixu[rs1] as i64) < (simm12 as i64) {
                            self.ixu[rd] = 1;
                        }
                        else {
                            self.ixu[rd] = 0;
                        }
                    }
                    0b011 => { //SLTIU: x[rd] = 1 if x[rs1] <u sext(immediate) else x[rd] = 0
                        println!("sltiu {},{},{}", REGNAME[rd], REGNAME[rs1], simm12 as i64);
                        if self.ixu[rs1] < simm12 {
                            self.ixu[rd] = 1;
                        }
                        else {
                            self.ixu[rd] = 0;
                        }
                    }
                    0b100 => { //XORI: x[rd] = x[rs1] ^ sext(immediate)
                        println!("xori {},{},{}", REGNAME[rd], REGNAME[rs1], simm12 as i64);
                        self.ixu[rd] = self.ixu[rs1] ^ simm12;
                    }
                    0b101 => {
                        //SRLI or SRAI
                        let funct7: u32 = getfield32!(inst, INST_FUNCT7_WID, INST_FUNCT7_POS);
                        //0 <= shamt <= 63, imm12[5:0] or inst[25:20] are used as shift value
                        let shamt = getfield32!(inst, INST_SHAMT_WID, INST_SHAMT_POS);
                        match funct7 {
                            0b0000000 => { //SRLI: x[rd] = x[rs1] >> shamt
                                //Inserts 0's in the vacant bits on left side
                                println!("srli {},{},{}", REGNAME[rd], REGNAME[rs1], shamt);
                                self.ixu[rd] = self.ixu[rs1] >> shamt;
                            }
                            0b0100000 => { //SRAI: x[rd] = sext(x[rs1] >> shamt)
                                //Inserts sign-bit(msb) in the vacant  bits on the left side to preserve the sign
                                println!("srai {},{},{}", REGNAME[rd], REGNAME[rs1], shamt);
                                self.ixu[rd] = signext_nto64(self.ixu[rs1] >> shamt, 64 - shamt as u64);
                            }
                            _ => panic!("Not handling this FUNCT7"),
                        }
                    }
                    0b110 => {
                        println!("ori {},{},{}", REGNAME[rd], REGNAME[rs1], simm12 as i64);
                        self.ixu[rd] = self.ixu[rs1] | simm12;
                    }
                    0b111 => {
                        println!("andi {},{},{}", REGNAME[rd], REGNAME[rs1], simm12 as i64);
                        self.ixu[rd] = self.ixu[rs1] & simm12;
                    }
                    _ => panic!("Not handling this Funct3"),
                };
            }
            _ => panic!("Illegal Instruction: 0b{:07b}", opcode),
        }

        Ok(())
    }

    /// Print values in all registers (x0-x31).
    pub fn print_registers(&self) {
        let mut output = String::from("");
        for i in (0..32).step_by(4) {
            output = format!(
                "{}\n{}",
                output,
                format!(
                    "{COLOR_GREEN}[{}]{COLOR_RESET} = {:#018x} \
                     {COLOR_GREEN}[{}]{COLOR_RESET} = {:#018x} \
                     {COLOR_GREEN}[{}]{COLOR_RESET} = {:#018x} \
                     {COLOR_GREEN}[{}]{COLOR_RESET} = {:#018x}",
                    REGNAME[i],
                    self.ixu[i],
                    REGNAME[i + 1],
                    self.ixu[i + 1],
                    REGNAME[i + 2],
                    self.ixu[i + 2],
                    REGNAME[i + 3],
                    self.ixu[i + 3],
                )
            );
        }

        print!("{COLOR_BLUE}[pc]{COLOR_RESET} = {:#018x}", self.pc);
        println!("{}", output);
        println!("----------------------------------------------\
        ---------------------------------------------------------")
    }

    fn pipeline(&self) -> Result<(), RiscvCpuError> {
        Ok(())
    }
}

fn read_bin(f: &String) -> Result<Vec<u8>, ErrorKind> {
    let mut content: Vec<u8> = Vec::new();
    let metadata = fs::metadata(f).expect("unable to get the metadata");
    if metadata.is_file() {
        // LATER: Better error handling whild reading the binary file
        let mut fbin = fs::File::open(f).unwrap();
        fbin.read_to_end(&mut content).unwrap();
        Ok(content)
    } else {
        println!("binary file invalid\n");
        // LATER: Right error type for metadata of not file
        Err(ErrorKind::Other)
    }
}

pub fn rvlator() {
    let args: Vec<String> = env::args().collect();
    let binfilepath = &args[1];
    let inststream = read_bin(binfilepath).expect("input binary missing");

    let mut cpu = RiscvCpu::new(inststream);

    for _ in 0..cpu.mem.len()/4 {
        let inst = cpu.fetch().unwrap();
        cpu.execute(inst).unwrap();
        cpu.print_registers();
        cpu.pc += 4;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prelog() -> RiscvCpu {
        let bin = read_bin(&String::from("test/bin/rvlatortest.bin")).unwrap();
        RiscvCpu::new(bin)
    }

    #[test]
    fn test_newcpu() {
        let mut cpu = prelog();
        let inst = cpu.fetch().unwrap();
        cpu.pc += 4;
        printinst!(inst);
    }

    #[test]
    fn test_validdecode() {
        let mut cpu = prelog();
        let inst = cpu.fetch().unwrap();
        cpu.pc += 4;
        assert_eq!((), cpu.execute(inst).unwrap());
    }

    #[test]
    fn test_invaliddecode1() {
        let mut cpu = prelog();
        assert_eq!(Err(RiscvCpuError::DecodeError), cpu.execute(0x00000000));
    }

    #[test]
    fn test_invaliddecode2() {
        let mut cpu = prelog();
        assert_eq!(Err(RiscvCpuError::DecodeError), cpu.execute(0x0000001f));
    }

    #[test]
    fn test_inst_addi_v1() {
        let mut cpu = prelog();
        // addi a0,zero,-4  (ffc00513)
        cpu.execute(0xffc00513).unwrap();
        assert_eq!(cpu.ixu[REG_A0], 0xfffffffffffffffc);
    }

    #[test]
    fn test_inst_addi_v2() {
        let mut cpu = prelog();
        // Above instruction flips the bits and
        // performs the NOT operation.
        
        // addi a0,zero,-4  (ffc00513)
        cpu.execute(0xffc00513).unwrap();
        // addi a0, a0, -1 (fff50513)
        cpu.execute(0xfff50513).unwrap();
        assert_eq!(cpu.ixu[REG_A0], 0xfffffffffffffffb);
    }

    #[test]
    fn test_inst_slti() {
        let mut cpu = prelog();
        // addi a1,zero,-5
        cpu.execute(0xffb00593).unwrap();
        //  slti a2, a1, -4 (ffc5a613)
        cpu.execute(0xffc5a613).unwrap();
        assert_eq!(cpu.ixu[REG_A2], 0x0000000000000001)
    }

    #[test]
    fn test_inst_slli() {
        let mut cpu = prelog();
        // addi a0, zero, -4  (ffc00513)
        cpu.execute(0xffc00513).unwrap();
        // slli a2, a0, 0x3c (03c51613)
        cpu.execute(0x03c51613).unwrap();
        assert_eq!(cpu.ixu[REG_A2], 0xc000000000000000);
    }

    #[test]
    fn test_inst_lui() {
        let mut cpu = prelog();
        // lui s4, 0xdead (0deada37)
        cpu.execute(0x0deada37).unwrap();
        assert_eq!(cpu.ixu[REG_S4], 0x000000000dead000);
    }

    #[test]
    fn test_inst_auipc() {
        let mut cpu = prelog();
        // Since pc == 0 for below instruction, for test increment it
        cpu.pc = 4;
        // auipc s3, 0xdead (0dead997)
        cpu.execute(0x0dead997).unwrap();
        assert_eq!(cpu.ixu[REG_S3], 0x000000000dead004);
    }
}
