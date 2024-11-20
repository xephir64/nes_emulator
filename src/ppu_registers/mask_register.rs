use bitflags::{bitflags, Flags};

bitflags! {
    // 7  bit  0
    // ---- ----
    // BGRs bMmG
    // |||| ||||
    // |||| |||+- Greyscale (0: normal color, 1: greyscale)
    // |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
    // |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
    // |||| +---- 1: Enable background rendering
    // |||+------ 1: Enable sprite rendering
    // ||+------- Emphasize red (green on PAL/Dendy)
    // |+-------- Emphasize green (red on PAL/Dendy)
    // +--------- Emphasize blue


    pub struct MaskRegister: u8 {
        const GREYSCALE = 0b_0000_0001;
        const LEFTMOST_BACKGROUND_8PX_SCREEN = 0b0000_0010;
        const LEFTMOST_SPRITES_8PX_SCREEN = 0b0000_0100;
        const BACKGROUND_RENDERING = 0b0000_1000;
        const SPRITE_RENDERING = 0b0001_0000;
        const EMPHASIZE_RED = 0b0010_0000;
        const EMPHASIZE_GREEN = 0b0100_0000;
        const EMPHASIZE_BLUE = 0b1000_0000;
    }
}

impl MaskRegister {
    pub fn new() -> Self {
        MaskRegister::from_bits_truncate(0b0000_0000)
    }

    pub fn is_grayscale(&self) -> bool {
        self.contains(MaskRegister::GREYSCALE)
    }

    pub fn leftmost_8pxl_background(&self) -> bool {
        self.contains(MaskRegister::LEFTMOST_BACKGROUND_8PX_SCREEN)
    }

    pub fn leftmost_8pxl_sprite(&self) -> bool {
        self.contains(MaskRegister::LEFTMOST_SPRITES_8PX_SCREEN)
    }

    pub fn show_background(&self) -> bool {
        self.contains(MaskRegister::BACKGROUND_RENDERING)
    }

    pub fn show_sprites(&self) -> bool {
        self.contains(MaskRegister::SPRITE_RENDERING)
    }

    pub fn update(&mut self, data: u8) {
        *self = MaskRegister::from_bits_retain(data);
    }
}
