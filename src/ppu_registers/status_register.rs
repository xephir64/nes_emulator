use bitflags::bitflags;

bitflags! {
    // 7  bit  0
    // ---- ----
    // VSOx xxxx
    // |||| ||||
    // |||+-++++- (PPU open bus or 2C05 PPU identifier)
    // ||+------- Sprite overflow flag
    // |+-------- Sprite 0 hit flag
    // +--------- Vblank flag, cleared on read. Unreliable; see below.
    pub struct StatusRegister: u8 {
        const UNUSED_0 = 0b0000_0001;
        const UNUSED_1 = 0b0000_0010;
        const UNUSED_2 = 0b0000_0100;
        const UNUSED_3 = 0b0000_1000;
        const IDENTIFIER = 0b0001_0000;
        const SPRITE_OVERFLOW_FLAG = 0b0010_0000;
        const SPRITE_ZERO_HIT_FLAG = 0b0100_0000;
        const VBLANK_FLAG = 0b1000_0000;
    }
}

impl StatusRegister {
    pub fn new() -> Self {
        StatusRegister::from_bits_truncate(0b0000_0000)
    }

    pub fn snapshot(&self) -> u8 {
        return self.bits();
    }

    pub fn reset_vblank_status(&mut self) {
        self.remove(StatusRegister::VBLANK_FLAG);
    }

    pub fn set_vblank_status(&mut self, condition: bool) {
        self.set(StatusRegister::VBLANK_FLAG, condition);
    }
}
