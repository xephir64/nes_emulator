use crate::{apu::Apu, cpu::Mem, joypad::Joypad, ppu::NesPPU, rom::Rom};

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const ROM: u16 = 0x8000;
const ROM_END: u16 = 0xFFFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

pub struct Bus<'call> {
    cpu_vram: [u8; 2048],
    prg_rom: Vec<u8>,
    ppu: NesPPU,
    apu: Apu,

    cycles: usize,
    gameloop_callback: Box<dyn FnMut(&NesPPU, &mut Joypad) + 'call>,
    joypad: Joypad,
}

impl<'a> Bus<'a> {
    pub fn new<'call, F>(rom: Rom, gameloop_callback: F) -> Bus<'call>
    where
        F: FnMut(&NesPPU, &mut Joypad) + 'call,
    {
        let ppu = NesPPU::new(rom.chr_rom, rom.screen_mirroring);

        Bus {
            cpu_vram: [0; 2048],
            prg_rom: rom.prg_rom,
            ppu: ppu,
            cycles: 0,
            gameloop_callback: Box::from(gameloop_callback),
            joypad: Joypad::new(),
            apu: Apu::new(),
        }
    }

    fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= 0x8000;
        if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            //mirror if needed
            addr = addr % 0x4000;
        }
        self.prg_rom[addr as usize]
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;

        for _ in 0..cycles {
            self.apu.tick();
        }

        let nmi_before = self.ppu.nmi_interrupt.is_some();
        self.ppu.tick(cycles * 3);
        let nmi_after = self.ppu.nmi_interrupt.is_some();

        if !nmi_before && nmi_after {
            (self.gameloop_callback)(&self.ppu, &mut self.joypad);
        }
    }

    pub fn poll_nmi_status(&mut self) -> Option<u8> {
        self.ppu.nmi_interrupt.take()
    }

    pub fn get_audio_samples(&mut self) -> Vec<f32> {
        self.apu.take_samples()
    }
}

impl<'a> Mem for Bus<'a> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize]
            }

            PPU_REGISTERS | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                panic!("Attempt to read from write-only PPU address {:x}", addr);
            }

            0x2002 => self.ppu.read_status(),

            0x2004 => self.ppu.read_oam_data(),

            0x2007 => self.ppu.read_data(),

            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_read(mirror_down_addr)
            }

            ROM..=ROM_END => self.read_prg_rom(addr),

            0x4000..=0x4015 => self.apu.read_register(addr),

            0x4016 => self.joypad.read(),

            0x4017 => {
                // ignore joypad 2
                0
            }

            _ => {
                println!("Ignoring mem access at {}", addr);
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b11111111111;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }

            PPU_REGISTERS => {
                self.ppu.write_to_ctrl(data);
            }

            0x2001 => {
                self.ppu.write_to_mask(data);
            }

            0x2002 => panic!("attempt to write to PPU status register"),

            0x2003 => {
                self.ppu.write_to_oam_addr(data);
            }

            0x2004 => {
                self.ppu.write_to_oam_data(data);
            }

            0x2005 => {
                self.ppu.write_to_scroll(data);
            }

            0x2006 => {
                self.ppu.write_to_ppu_addr(data);
            }

            0x2007 => {
                self.ppu.write_to_data(data);
            }

            0x4000..=0x4013 | 0x4015 => {
                self.apu.write_register(addr, data);
            }

            0x4014 => {
                let mut buffer: [u8; 256] = [0; 256];
                let hi: u16 = (data as u16) << 8;
                for i in 0..256u16 {
                    buffer[i as usize] = self.mem_read(hi + i);
                }

                self.ppu.write_oam_dma(&buffer);

                // todo: handle this eventually
                // let add_cycles: u16 = if self.cycles % 2 == 1 { 514 } else { 513 };
                // self.tick(add_cycles); //todo this will cause weird effects as PPU will have 513/514 * 3 ticks
            }

            0x4016 => self.joypad.write(data),

            0x4017 => {
                // ignore joypad 2;
            }

            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_write(mirror_down_addr, data);
            }

            ROM..=ROM_END => panic!("Attempt to write to Cartridge ROM space"),

            _ => {
                println!("Ignoring mem write-access at {}", addr);
            }
        }
    }
}
