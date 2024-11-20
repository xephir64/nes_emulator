pub struct ScrollRegister {
    scroll_x: u8,
    scroll_y: u8,
    w_latch: bool,
}

impl ScrollRegister {
    pub fn new() -> Self {
        ScrollRegister {
            scroll_x: 0b0000_0000,
            scroll_y: 0b0000_0000,
            w_latch: false,
        }
    }

    pub fn reset_latch(&mut self) {
        self.w_latch = false;
    }
}
