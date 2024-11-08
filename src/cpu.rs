use crate::opcode::OPCODES_MAP;

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    memory: [u8; 0xFFFF],
}

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

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0; 0xFFFF],
        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
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
    }

    fn asl(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.update_carry_flag(value >> 7 == 1);

        let data = value << 1;
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);

        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn inx(&mut self) {
        if self.register_x != u8::MAX {
            self.register_x = self.register_x + 1;
        } else {
            self.register_x = 0;
        }

        self.update_zero_and_negative_flags(self.register_x);
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

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.status = self.status | 0b0000_0010; // set 2nd byte (aka Zero flag) to 1 if result = 0
        } else {
            self.status = self.status & 0b1111_1101; // set 2nd byte (aka Zero flag) to 0 if result != 0
        }

        if result & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000; // set 7th byte (aka Negative flag) to 1 if result is negative
        } else {
            self.status = self.status & 0b0111_1111; // set 7th byte (aka Negative flag) to 1 if result is positive
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
        self.status = 0;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run()
    }

    pub fn run(&mut self) {
        loop {
            let opscode = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let program_counter_state = self.program_counter;

            let opcode = OPCODES_MAP.get(&opscode).unwrap();

            match opcode.mnemonic {
                "ADC" => {
                    self.adc(&opcode.addr);
                }

                "AND" => {
                    self.and(&opcode.addr);
                }

                "ASL" => {
                    if opcode.op_code == 0x0A {
                        self.asl_accumulator();
                    } else {
                        self.asl(&opcode.addr);
                    }
                }

                "LDA" => {
                    self.lda(&opcode.addr);
                }

                "STA" => {
                    self.sta(&opcode.addr);
                }

                "TAX" => {
                    self.tax();
                }

                "INX" => {
                    self.inx();
                }

                "BRK" => {
                    return;
                }

                _ => todo!(),
            }

            if program_counter_state == self.program_counter {
                self.program_counter += (opcode.byte - 1) as u16;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x0a, 0xaa, 0x00]);

        assert_eq!(cpu.register_x, 10)
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_lda_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_adc_immediate_basic_addition() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x05;
        cpu.load_and_run(vec![0xA9, 0x05, 0x69, 0x03, 0x00]); // LDA #$05 ADC #$03 BRK

        assert_eq!(cpu.register_a, 0x08); // 5 + 3 = 8
        assert!(cpu.status & 0b0000_0001 == 0);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_adc_with_carry_set() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0xFF, 0x69, 0x01, 0xA9, 0x05, 0x69, 0x03, 0x00]); // LDA #$FF ADC #$01 LDA #$05 ADC #$03 BRK

        assert_eq!(cpu.register_a, 0x09);
        assert!(cpu.status & 0b0000_0001 == 0);
    }

    #[test]
    fn test_adc_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x50, 0x69, 0x50, 0x00]); // LDA #$50 ADC #$50 BRK

        assert_eq!(cpu.register_a, 0xa0); // 0x50 + 0x50 = 0xa0
        assert!(cpu.status & 0b0100_0000 != 0);
        assert!(cpu.status & 0b1000_0000 != 0);
    }

    #[test]
    fn test_and() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x11, 0x29, 0x10, 0x00]); // LDA $#11 AND $#10 BRK

        assert_eq!(cpu.register_a, 0x10); // 0x11 & 0x10 = 0x10
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_and_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0xCC, 0x29, 0xAA, 0x00]); // LDA #$CC AND #$AA BRK

        assert_eq!(cpu.register_a, 0x88);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 != 0);
    }

    #[test]
    fn test_asl_accumulator() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x4D, 0x0A, 0x00]); // LDA #$4D ASL BRK

        assert_eq!(cpu.register_a, 0x9A);
        assert!(cpu.status & 0b0000_0001 == 0);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_asl_zero_page() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x81);
        cpu.load_and_run(vec![0x06, 0x10, 0x00]); // ASL $10 BRK

        assert_eq!(cpu.mem_read(0x10), 0x02);
        assert!(cpu.status & 0b0000_0001 != 0);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 == 0);
    }
}
