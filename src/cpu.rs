use std::collections::HashMap;

use crate::{
    bus::Bus,
    opcode::{self},
};

const STACK: u16 = 0x0100;
const STACK_RESET: u8 = 0xfd;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}
pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub bus: Bus,
}

pub trait Mem {
    fn mem_read(&self, addr: u16) -> u8;

    fn mem_write(&mut self, addr: u16, data: u8);

    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
}

impl CPU {
    pub fn new(bus: Bus) -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            stack_pointer: STACK_RESET,
            bus,
        }
    }

    pub fn mem_read(&self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data)
    }

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }

    fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.mem_read((STACK as u16) + self.stack_pointer as u16)
    }

    fn stack_push(&mut self, data: u8) {
        self.mem_write((STACK as u16) + self.stack_pointer as u16, data);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1)
    }

    fn stack_push_u16(&mut self, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.stack_push(hi);
        self.stack_push(lo);
    }

    fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;

        hi << 8 | lo
    }

    fn jump_to_branch(&mut self, condition: bool) {
        if condition {
            let jump = self.mem_read(self.program_counter) as i8;
            let jump_addr = self
                .program_counter
                .wrapping_add(1)
                .wrapping_add(jump as u16);

            self.program_counter = jump_addr;
        }
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value_1 = self.register_a;
        let value_2 = self.mem_read(addr);
        let carry = self.status & 0b0000_0001;

        let sum = value_1 as u16 + value_2 as u16 + carry as u16;

        let enable_carry = sum > 255;

        let result = sum as u8;
        let is_overflow = (value_2 ^ result) & (result ^ self.register_a) & 0x80 != 0;

        self.register_a = sum as u8;
        self.update_carry_flag(enable_carry);
        self.update_overflow_flag(is_overflow);
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let a = self.register_a;
        let m = self.mem_read(addr);

        self.register_a = a & m;

        self.update_zero_and_negative_flags(self.register_a);
    }

    fn asl_accumulator(&mut self) {
        self.update_carry_flag(self.register_a >> 7 == 1);
        self.register_a = self.register_a << 1;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn asl(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.update_carry_flag(value >> 7 == 1);

        let data = value << 1;
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
    }

    fn bcc(&mut self) {
        let is_carry_flag = self.status & 0b0000_0001 == 0;
        self.jump_to_branch(is_carry_flag);
    }

    fn bcs(&mut self) {
        let is_carry_flag = self.status & 0b0000_0001 == 1;
        self.jump_to_branch(is_carry_flag);
    }

    fn beq(&mut self) {
        self.jump_to_branch(((self.status & 0b0000_0010) >> 1) == 1);
    }

    fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mem_val = self.mem_read(addr);

        if self.register_a & mem_val == 0 {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }

        self.update_overflow_flag(mem_val >> 6 == 1);
        self.update_negative_flag(mem_val >> 7 == 1);
    }

    fn bmi(&mut self) {
        self.jump_to_branch(((self.status & 0b1000_0000) >> 7) == 1);
    }

    fn bne(&mut self) {
        self.jump_to_branch(((self.status & 0b0000_0010) >> 1) == 0);
    }

    fn bpl(&mut self) {
        self.jump_to_branch(((self.status & 0b1000_0000) >> 7) == 0);
    }

    fn bvc(&mut self) {
        self.jump_to_branch(((self.status & 0b0100_0000) >> 6) == 0);
    }

    fn bvs(&mut self) {
        self.jump_to_branch(((self.status & 0b0100_0000) >> 6) == 1);
    }

    fn clc(&mut self) {
        self.update_carry_flag(false);
    }

    fn cld(&mut self) {
        self.status = self.status & 0b1111_0111;
    }

    fn cli(&mut self) {
        self.status = self.status & 0b1111_1011;
    }

    fn clv(&mut self) {
        self.update_overflow_flag(false);
    }

    fn cmp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        if self.register_a >= value {
            self.update_carry_flag(true);
        } else {
            self.update_carry_flag(false);
        }

        self.update_zero_and_negative_flags(self.register_a.wrapping_sub(value));
    }

    fn cpx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        if value <= self.register_x {
            self.update_carry_flag(true);
        } else {
            self.update_carry_flag(false);
        }

        self.update_zero_and_negative_flags(self.register_x.wrapping_sub(value));
    }

    fn cpy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        if self.register_y >= value {
            self.update_carry_flag(true);
        } else {
            self.update_carry_flag(false);
        }

        self.update_zero_and_negative_flags(self.register_y.wrapping_sub(value));
    }

    fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut value = self.mem_read(addr);

        value = value.wrapping_sub(1);
        self.mem_write(addr, value);
        self.update_zero_and_negative_flags(value);
    }

    fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn dey(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let a = self.register_a;
        let m = self.mem_read(addr);

        self.register_a = a ^ m;

        self.update_zero_and_negative_flags(self.register_a);
    }

    fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut value = self.mem_read(addr);

        value = value.wrapping_add(1);

        self.mem_write(addr, value);
        self.update_zero_and_negative_flags(value);
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn jmp_abs(&mut self) {
        let mem_address = self.mem_read_u16(self.program_counter);
        self.program_counter = mem_address;
    }

    fn jmp_indirect(&mut self) {
        let mem_address = self.mem_read_u16(self.program_counter);

        let indirect_ref = if mem_address & 0x00FF == 0x00FF {
            let lo = self.mem_read(mem_address);
            let hi = self.mem_read(mem_address & 0xFF00);
            (hi as u16) << 8 | (lo as u16)
        } else {
            self.mem_read_u16(mem_address)
        };

        self.program_counter = indirect_ref;
    }

    fn jsr(&mut self) {
        self.stack_push_u16(self.program_counter + 2 - 1);
        let mem_address = self.mem_read_u16(self.program_counter);
        self.program_counter = mem_address
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_x = value;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_y = value;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn lsr_accumulator(&mut self) {
        self.update_carry_flag(self.register_a & 1 == 1);
        self.register_a = self.register_a >> 1;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn lsr(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.update_carry_flag(value & 1 == 1);

        let data = value >> 1;
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let a = self.register_a;
        let m = self.mem_read(addr);

        self.register_a = a | m;

        self.update_zero_and_negative_flags(self.register_a);
    }

    fn pha(&mut self) {
        self.stack_push(self.register_a);
    }

    fn php(&mut self) {
        self.stack_push(self.status | 0b0011_0000);
    }

    fn pla(&mut self) {
        let value = self.stack_pop();

        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn plp(&mut self) {
        let mut flags = self.stack_pop();
        flags = flags & 0b1110_1111; // unset BREAK
        flags = flags | 0b0010_0000; // set BREAK 2
        self.status = flags;
    }

    fn rol_accumulator(&mut self) {
        let old_carry_flag = self.status & 1 == 1;

        self.update_carry_flag(self.register_a >> 7 == 1);
        self.register_a = self.register_a << 1;

        if old_carry_flag {
            self.register_a = self.register_a | 1;
        }

        self.update_zero_and_negative_flags(self.register_a);
    }

    fn rol(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let old_carry_flag = self.status & 1 == 1;

        self.update_carry_flag(value >> 7 == 1);

        let mut data = value << 1;
        if old_carry_flag {
            data = data | 1;
        }

        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
    }

    fn ror_accumulator(&mut self) {
        let old_carry_flag = self.status & 1 == 1;

        self.update_carry_flag(self.register_a & 1 == 1);
        self.register_a = self.register_a >> 1;

        if old_carry_flag {
            self.register_a = self.register_a | 0b1000_0000;
        }

        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ror(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let old_carry_flag = self.status & 1 == 1;

        self.update_carry_flag(value & 1 == 1);

        let mut data = value >> 1;

        if old_carry_flag {
            data = data | 0b1000_0000;
        }

        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
    }

    fn rti(&mut self) {
        let mut flags = self.stack_pop();
        flags = flags & 0b1110_1111; // unset BREAK
        flags = flags | 0b0010_0000; // set BREAK 2
        self.status = flags;

        self.program_counter = self.stack_pop_u16();
    }

    fn rts(&mut self) {
        self.program_counter = self.stack_pop_u16() + 1;
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let a = self.register_a;
        let m = self.mem_read(addr);
        let carry = self.status & 0b0000_0001;

        let sub = (m as i8).wrapping_neg().wrapping_sub(1) as u8; // -B = !B - 1

        let sum = a as u16 + sub as u16 + carry as u16; // A + (-B) + C

        let enable_carry = sum > 255;

        let result = sum as u8;
        let is_overflow = (m ^ result) & (result ^ self.register_a) & 0x80 != 0;

        self.register_a = sum as u8;
        self.update_carry_flag(enable_carry);
        self.update_overflow_flag(is_overflow);
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn sec(&mut self) {
        self.update_carry_flag(true);
    }

    fn sed(&mut self) {
        self.status = self.status | 0b0000_1000;
    }

    fn sei(&mut self) {
        self.status = self.status | 0b0000_0100;
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);

        self.update_zero_and_negative_flags(self.register_a);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);

        self.update_zero_and_negative_flags(self.register_x);
    }

    fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);

        self.update_zero_and_negative_flags(self.register_y);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn tsx(&mut self) {
        self.register_x = self.stack_pointer;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn txs(&mut self) {
        self.stack_pointer = self.register_x;
    }

    fn tya(&mut self) {
        self.register_a = self.register_y;
    }

    fn update_carry_flag(&mut self, enable: bool) {
        if enable {
            self.status = self.status | 0b0000_0001;
        } else {
            self.status = self.status & 0b1111_1110;
        }
    }

    fn update_overflow_flag(&mut self, enable: bool) {
        if enable {
            self.status = self.status | 0b0100_0000;
        } else {
            self.status = self.status & 0b1011_1111;
        }
    }

    fn update_negative_flag(&mut self, enable: bool) {
        if enable {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
        }
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.status = self.status | 0b0000_0010; // set 2nd byte (aka Zero flag) to 1 if result = 0
        } else {
            self.status = self.status & 0b1111_1101; // set 2nd byte (aka Zero flag) to 0 if result != 0
        }

        if result & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000; // set 7th byte (aka Negative flag) to 1 if result is negative
        } else {
            self.status = self.status & 0b0111_1111; // set 7th byte (aka Negative flag) to 0 if result is positive
        }
    }

    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,

            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,

            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),

            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }

            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }

            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }

            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }

            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);

                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }

            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }

            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = 0;
        self.stack_pointer = STACK_RESET;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        // self.bus.cpu_vram[0x0600..(0x0600 + program.len())].copy_from_slice(&program[..]);
        // let addr = self.mem_read_u16(0x0600..(0x0600 + program.len()));
        // self.mem_write_u16(addr, &program[..]);
        for i in 0..(program.len() as u16) {
            self.mem_write(0x0600 + i, program[i as usize]);
        }

        self.mem_write_u16(0xFFFC, 0x0600);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run()
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        let ref opcodes: HashMap<u8, &'static opcode::OpCode> = *opcode::OPCODES_MAP;

        loop {
            let opscode = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let program_counter_state = self.program_counter;

            let instruction = opcodes
                .get(&opscode)
                .expect(&format!("OpCode {:x} is not recognized", opscode));

            println!(
                "Instruction: {}, OpCode: {:#04x}, CPU Status: {:08b}, PC: {}",
                instruction.mnemonic, instruction.op_code, self.status, self.program_counter
            );

            match instruction.mnemonic {
                "ADC" => {
                    self.adc(&instruction.addr);
                }

                "AND" => {
                    self.and(&instruction.addr);
                }

                "ASL" => {
                    if instruction.op_code == 0x0A {
                        self.asl_accumulator();
                    } else {
                        self.asl(&instruction.addr);
                    }
                }

                "BCC" => {
                    self.bcc();
                }

                "BCS" => {
                    self.bcs();
                }

                "BEQ" => {
                    self.beq();
                }

                "BIT" => {
                    self.bit(&instruction.addr);
                }

                "BMI" => {
                    self.bmi();
                }

                "BNE" => {
                    self.bne();
                }

                "BPL" => {
                    self.bpl();
                }

                "BVC" => {
                    self.bvc();
                }

                "BVS" => {
                    self.bvs();
                }

                "CLC" => {
                    self.clc();
                }

                "CLD" => {
                    self.cld();
                }

                "CLI" => {
                    self.cli();
                }

                "CLV" => {
                    self.clv();
                }

                "CMP" => {
                    self.cmp(&instruction.addr);
                }

                "CPX" => {
                    self.cpx(&instruction.addr);
                }

                "CPY" => {
                    self.cpy(&instruction.addr);
                }

                "DEC" => {
                    self.dec(&instruction.addr);
                }

                "DEX" => {
                    self.dex();
                }

                "DEY" => {
                    self.dey();
                }

                "EOR" => {
                    self.eor(&instruction.addr);
                }

                "INC" => {
                    self.inc(&instruction.addr);
                }

                "INX" => {
                    self.inx();
                }

                "INY" => {
                    self.iny();
                }

                "JMP" => {
                    if instruction.op_code == 0x4C {
                        self.jmp_abs();
                    } else {
                        self.jmp_indirect();
                    }
                }

                "JSR" => {
                    self.jsr();
                }

                "LDA" => {
                    self.lda(&instruction.addr);
                }

                "LDX" => {
                    self.ldx(&instruction.addr);
                }

                "LDY" => {
                    self.ldy(&instruction.addr);
                }

                "LSR" => {
                    if instruction.op_code == 0x4A {
                        self.lsr_accumulator();
                    } else {
                        self.lsr(&instruction.addr);
                    }
                }

                "NOP" => {}

                "ORA" => {
                    self.ora(&instruction.addr);
                }

                "PHA" => {
                    self.pha();
                }

                "PHP" => {
                    self.php();
                }

                "PLA" => {
                    self.pla();
                }

                "PLP" => {
                    self.plp();
                }

                "ROL" => {
                    if instruction.op_code == 0x2A {
                        self.rol_accumulator();
                    } else {
                        self.rol(&instruction.addr);
                    }
                }

                "ROR" => {
                    if instruction.op_code == 0x6A {
                        self.ror_accumulator();
                    } else {
                        self.ror(&instruction.addr);
                    }
                }

                "RTI" => {
                    self.rti();
                }

                "RTS" => {
                    self.rts();
                }

                "SBC" => {
                    self.sbc(&instruction.addr);
                }

                "SEC" => {
                    self.sec();
                }

                "SED" => {
                    self.sed();
                }

                "SEI" => {
                    self.sei();
                }

                "STA" => {
                    self.sta(&instruction.addr);
                }

                "STX" => {
                    self.stx(&instruction.addr);
                }

                "STY" => {
                    self.sty(&instruction.addr);
                }

                "TAX" => {
                    self.tax();
                }

                "TAY" => {
                    self.tay();
                }

                "TSX" => {
                    self.tsx();
                }

                "TXA" => {
                    self.txa();
                }

                "TXS" => {
                    self.txs();
                }

                "TYA" => {
                    self.tya();
                }

                "BRK" => {
                    return;
                }

                _ => todo!(),
            }

            if program_counter_state == self.program_counter {
                self.program_counter += (instruction.len - 1) as u16;
            }

            callback(self);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x0a, 0xaa, 0x00]);

        assert_eq!(cpu.register_x, 10)
    }

    #[test]
    fn test_5_ops_working_together() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_lda_from_memory() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_adc_immediate_basic_addition() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.register_a = 0x05;
        cpu.load_and_run(vec![0xA9, 0x05, 0x69, 0x03, 0x00]); // LDA #$05 ADC #$03 BRK

        assert_eq!(cpu.register_a, 0x08); // 5 + 3 = 8
        assert!(cpu.status & 0b0000_0001 == 0);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_adc_with_carry_set() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xA9, 0xFF, 0x69, 0x01, 0xA9, 0x05, 0x69, 0x03, 0x00]); // LDA #$FF ADC #$01 LDA #$05 ADC #$03 BRK

        assert_eq!(cpu.register_a, 0x09);
        assert!(cpu.status & 0b0000_0001 == 0);
    }

    #[test]
    fn test_adc_overflow() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xA9, 0x50, 0x69, 0x50, 0x00]); // LDA #$50 ADC #$50 BRK

        assert_eq!(cpu.register_a, 0xa0); // 0x50 + 0x50 = 0xa0
        assert!(cpu.status & 0b0100_0000 != 0);
        assert!(cpu.status & 0b1000_0000 != 0);
    }

    #[test]
    fn test_and() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xA9, 0x11, 0x29, 0x10, 0x00]); // LDA $#11 AND $#10 BRK

        assert_eq!(cpu.register_a, 0x10); // 0x11 & 0x10 = 0x10
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_and_negative_flag() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xA9, 0xCC, 0x29, 0xAA, 0x00]); // LDA #$CC AND #$AA BRK

        assert_eq!(cpu.register_a, 0x88);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 != 0);
    }

    #[test]
    fn test_asl_accumulator() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xA9, 0x4D, 0x0A, 0x00]); // LDA #$4D ASL BRK

        assert_eq!(cpu.register_a, 0x9A);
        assert!(cpu.status & 0b0000_0001 == 0);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 != 0);
    }

    #[test]
    fn test_asl_zero_page() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.mem_write(0x10, 0x81);
        cpu.load_and_run(vec![0x06, 0x10, 0x00]); // ASL $10 BRK

        assert_eq!(cpu.mem_read(0x10), 0x02);
        assert!(cpu.status & 0b0000_0001 != 0);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0x24_bit_zero_flag_set() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.mem_write(0x10, 0x92); // 0b1001_0010 so negative should be set
        cpu.load_and_run(vec![0xA9, 0x00, 0x24, 0x10, 0x00]); // LDA #$00, BIT $10 BRK

        assert!(cpu.status & 0b0000_0010 != 0);
        assert!(cpu.status & 0b0100_0000 == 0);
        assert!(cpu.status & 0b1000_0000 != 0);
    }

    #[test]
    fn test_0x24_bit_zero_flag_clear() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.mem_write(0x10, 0x01); // 0b0000_0001
        cpu.load_and_run(vec![0xA9, 0x01, 0x24, 0x10, 0x00]); // LDA #$01, BIT $10 BRK

        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b0100_0000 == 0);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_sbc_immediate_basic_subtraction() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xA9, 0x10, 0xE9, 0x05, 0x00]); // LDA #$10 SBC #$05 BRK

        assert_eq!(cpu.register_a, 0x0A);
        assert!(cpu.status & 0b0000_0001 == 1);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_sbc_with_borrow() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xA9, 0x05, 0xE9, 0x10, 0x00]); // LDA #$05 SBC #$10 BRK

        assert_eq!(cpu.register_a, 0xF4); // A=$f4
        assert!(cpu.status & 0b0000_0001 == 0); // NV-BDIZC
                                                // 10110000
        assert!(cpu.status & 0b1000_0000 != 0);
    }

    #[test]
    fn test_sbc() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xA9, 0x05, 0xE9, 0x05, 0x00]); // LDA #$05 SBC #$05 BRK

        assert_eq!(cpu.register_a, 0xff);
        assert!(cpu.status & 0b0000_0001 == 0);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 != 0);
    }

    #[test]
    fn test_ora() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xA9, 0x12, 0x09, 0x08, 0x00]); // LDA #$12 ORA #$08 BRK

        assert_eq!(cpu.register_a, 0x1A); // 0x12 | 0x08 = 0x1A
    }

    #[test]
    fn test_eor() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xA9, 0x15, 0x49, 0x0F, 0x00]); // LDA #$15 EOR #$0F BRK

        assert_eq!(cpu.register_a, 0x1A); // 0x15 ^ 0x0F = 0x1A
    }

    #[test]
    fn test_cmp_equal() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xA9, 0x05, 0xC9, 0x05, 0x00]); // LDA #$05 CMP #$05 BRK

        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0001 != 0);
        assert!(cpu.status & 0b0000_0010 != 0);
    }

    #[test]
    fn test_lsr_accumulator() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xA9, 0x02, 0x4A, 0x00]); // LDA #$02 LSR BRK

        assert_eq!(cpu.register_a, 0x01);
        assert!(cpu.status & 0b0000_0001 == 0);
        assert!(cpu.status & 0b0000_0010 == 0);
    }

    #[test]
    fn test_lsr_zero_page() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.mem_write(0x10, 0x01);
        cpu.load_and_run(vec![0x46, 0x10, 0x00]); // LSR $10 BRK

        assert_eq!(cpu.mem_read(0x10), 0x00);
        assert!(cpu.status & 0b0000_0001 != 0);
        assert!(cpu.status & 0b0000_0010 != 0);
    }

    #[test]
    fn test_rol_accumulator() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x81, 0x2A, 0x00]); // LDA #$81 ROL BRK

        assert_eq!(cpu.register_a, 0x02);
        assert!(cpu.status & 0b0000_0001 != 0);
        assert!(cpu.status & 0b0000_0010 == 0);
    }

    #[test]
    fn test_rol_with_carry_in() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0x38, 0xA9, 0x40, 0x2A, 0x00]); // SEC LDA #$40 ROL BRK

        assert_eq!(cpu.register_a, 0x81);
        assert!(cpu.status & 0b0000_0001 == 0);
    }

    #[test]
    fn test_ror_accumulator() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x01, 0x6a, 0x00]); // LDA #$01 ROR BRK

        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.status & 0b0000_0001 != 0);
        assert!(cpu.status & 0b0000_0010 != 0);
    }

    #[test]
    fn test_ror_with_carry_in() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0x38, 0xa9, 0x02, 0x6A, 0x00]); // SEC LDA #$02 ROR BRK

        assert_eq!(cpu.register_a, 0x81);
        assert!(cpu.status & 0b0000_0001 == 0);
        assert!(cpu.status & 0b1000_0000 != 0);
    }

    #[test]
    fn test_pha_pla() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xA9, 0x45, 0x48, 0xA9, 0x00, 0x68, 0x00]); // LDA #$45 PHA LDA #$00 PLA BRK

        assert_eq!(cpu.register_a, 0x45);
    }

    #[test]
    fn test_jmp_absolute() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0x4C, 0x10, 0x00, 0x00]); // JMP $0010 BRK

        assert_eq!(cpu.program_counter, 0x0011);
    }
}
