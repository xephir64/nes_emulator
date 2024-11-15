use std::collections::HashMap;

use crate::{
    cpu::{AddressingMode, CPU},
    opcode,
};

pub fn get_absolute_address(cpu: &CPU, mode: &AddressingMode, addr: u16) -> u16 {
    match mode {
        AddressingMode::ZeroPage => cpu.mem_read(addr) as u16,

        AddressingMode::Absolute => cpu.mem_read_u16(addr),

        AddressingMode::ZeroPage_X => {
            let pos = cpu.mem_read(addr);
            let addr = pos.wrapping_add(cpu.register_x) as u16;
            addr
        }
        AddressingMode::ZeroPage_Y => {
            let pos = cpu.mem_read(addr);
            let addr = pos.wrapping_add(cpu.register_y) as u16;
            addr
        }

        AddressingMode::Absolute_X => {
            let base = cpu.mem_read_u16(addr);
            let addr = base.wrapping_add(cpu.register_x as u16);
            addr
        }
        AddressingMode::Absolute_Y => {
            let base = cpu.mem_read_u16(addr);
            let addr = base.wrapping_add(cpu.register_y as u16);
            addr
        }

        AddressingMode::Indirect_X => {
            let base = cpu.mem_read(addr);

            let ptr: u8 = (base as u8).wrapping_add(cpu.register_x);
            let lo = cpu.mem_read(ptr as u16);
            let hi = cpu.mem_read(ptr.wrapping_add(1) as u16);
            (hi as u16) << 8 | (lo as u16)
        }
        AddressingMode::Indirect_Y => {
            let base = cpu.mem_read(addr);

            let lo = cpu.mem_read(base as u16);
            let hi = cpu.mem_read((base as u8).wrapping_add(1) as u16);
            let deref_base = (hi as u16) << 8 | (lo as u16);
            let deref = deref_base.wrapping_add(cpu.register_y as u16);
            deref
        }

        _ => {
            panic!("mode {:?} is not supported", mode);
        }
    }
}

pub fn trace(cpu: &CPU) -> String {
    let ref opcodes: HashMap<u8, &'static opcode::OpCode> = *opcode::OPCODES_MAP;

    let pc = cpu.program_counter;
    let code = cpu.mem_read(pc);
    let instruction = opcodes.get(&code).unwrap();

    let hex_dump: String;
    let asm_dump: String;

    match instruction.len {
        1 => {
            hex_dump = format!("{:04X}  {:02X}        ", pc, code);
            asm_dump = format!("{}", instruction.mnemonic);
        }
        2 => {
            let addr = cpu.mem_read(pc + 1);
            hex_dump = format!("{:04X}  {:02X} {:02X}     ", pc, code, addr);
            asm_dump = match instruction.addr {
                AddressingMode::NoneAddressing => format!("{}", instruction.mnemonic),
                AddressingMode::Immediate => format!("{} #${:02X}", instruction.mnemonic, addr),
                AddressingMode::ZeroPage => {
                    let mem_addr = get_absolute_address(cpu, &instruction.addr, pc + 1);
                    let stored_value = cpu.mem_read(mem_addr);
                    format!(
                        "{} ${:02X} = {:02X}",
                        instruction.mnemonic, mem_addr, stored_value
                    )
                }
                AddressingMode::ZeroPage_X => {
                    let mem_addr = get_absolute_address(cpu, &instruction.addr, pc + 1);
                    let stored_value = cpu.mem_read(mem_addr);
                    format!(
                        "{} ${:02X},X @ {:02X} = {:02X}",
                        instruction.mnemonic, addr, mem_addr, stored_value
                    )
                }
                AddressingMode::ZeroPage_Y => {
                    let mem_addr = get_absolute_address(cpu, &instruction.addr, pc + 1);
                    let stored_value = cpu.mem_read(mem_addr);
                    format!(
                        "{} ${:02X},Y @ {:02X} = {:02X}",
                        instruction.mnemonic, addr, mem_addr, stored_value
                    )
                }
                AddressingMode::Indirect_X => {
                    let mem_addr = get_absolute_address(cpu, &instruction.addr, pc + 1);
                    let stored_value = cpu.mem_read(mem_addr);
                    format!(
                        "{} (${:02X},X) @ {:02X} = {:04X} = {:02X}",
                        instruction.mnemonic,
                        addr,
                        addr.wrapping_add(cpu.register_x),
                        mem_addr,
                        stored_value
                    )
                }
                AddressingMode::Indirect_Y => {
                    let mem_addr = get_absolute_address(cpu, &instruction.addr, pc + 1);
                    let stored_value = cpu.mem_read(mem_addr);
                    format!(
                        "{} (${:02X}),Y = {:04X} @ {:04X} = {:02X}",
                        instruction.mnemonic,
                        addr,
                        mem_addr.wrapping_sub(cpu.register_y as u16),
                        mem_addr,
                        stored_value
                    )
                }
                _ => panic!(
                    "Unsupported addressing mode 2 for instruction {}",
                    instruction.mnemonic
                ),
            };
        }
        3 => {
            let low = cpu.mem_read(pc + 1);
            let high = cpu.mem_read(pc + 2);
            let addr = cpu.mem_read_u16(pc + 1);

            hex_dump = format!("{:04X}  {:02X} {:02X} {:02X}  ", pc, code, low, high);
            asm_dump = match instruction.addr {
                AddressingMode::NoneAddressing => {
                    format!(
                        "{} ${:04X}",
                        instruction.mnemonic,
                        ((high as u16) << 8) | (low as u16)
                    )
                }
                AddressingMode::Absolute => {
                    let mem_addr = get_absolute_address(cpu, &instruction.addr, pc + 1);
                    let stored_value = cpu.mem_read(mem_addr);
                    format!("${:04X} = {:02X}", mem_addr, stored_value)
                }
                AddressingMode::Absolute_X => {
                    let mem_addr = get_absolute_address(cpu, &instruction.addr, pc + 1);
                    let stored_value = cpu.mem_read(mem_addr);
                    format!("${:04X},X @ {:04X} = {:02X}", addr, mem_addr, stored_value)
                }
                AddressingMode::Absolute_Y => {
                    let mem_addr = get_absolute_address(cpu, &instruction.addr, pc + 1);
                    let stored_value = cpu.mem_read(mem_addr);
                    format!("${:04X},Y @ {:04X} = {:02X}", addr, mem_addr, stored_value)
                }
                _ => panic!(
                    "Unsupported addressing mode 3 for instruction {}",
                    instruction.mnemonic
                ),
            }
        }
        _ => panic!("Unexpected instruction length"),
    }

    let a = cpu.register_a;
    let x = cpu.register_x;
    let y = cpu.register_y;
    let p = cpu.status;
    let sp = cpu.stack_pointer;

    format!(
        "{}{:<31} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
        hex_dump, asm_dump, a, x, y, p, sp
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::bus::Bus;
    use crate::cpu::{Mem, CPU};
    use crate::rom::test::test_rom;

    #[test]
    fn test_format_trace() {
        let mut bus = Bus::new(test_rom());
        bus.mem_write(100, 0xa2);
        bus.mem_write(101, 0x01);
        bus.mem_write(102, 0xca);
        bus.mem_write(103, 0x88);
        bus.mem_write(104, 0x00);

        let mut cpu = CPU::new(bus);
        cpu.program_counter = 0x64;
        cpu.register_a = 1;
        cpu.register_x = 2;
        cpu.register_y = 3;
        let mut result: Vec<String> = vec![];
        cpu.run_with_callback(|cpu| {
            result.push(trace(cpu));
        });
        assert_eq!(
            "0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD",
            result[0]
        );
        assert_eq!(
            "0066  CA        DEX                             A:01 X:01 Y:03 P:24 SP:FD",
            result[1]
        );
        assert_eq!(
            "0067  88        DEY                             A:01 X:00 Y:03 P:26 SP:FD",
            result[2]
        );
    }

    #[test]
    fn test_format_mem_access() {
        let mut bus = Bus::new(test_rom());
        // ORA ($33), Y
        bus.mem_write(100, 0x11);
        bus.mem_write(101, 0x33);

        //data
        bus.mem_write(0x33, 00);
        bus.mem_write(0x34, 04);

        //target cell
        bus.mem_write(0x400, 0xAA);

        let mut cpu = CPU::new(bus);
        cpu.program_counter = 0x64;
        cpu.register_y = 0;
        let mut result: Vec<String> = vec![];
        cpu.run_with_callback(|cpu| {
            result.push(trace(cpu));
        });
        assert_eq!(
            "0064  11 33     ORA ($33),Y = 0400 @ 0400 = AA  A:00 X:00 Y:00 P:24 SP:FD",
            result[0]
        );
    }
}
