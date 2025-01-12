pub struct TriangleChannel {
    enabled: bool,
    length_counter: u8,
    timer: u16,
    timer_reload: u16,
    linear_counter: u8,
    linear_counter_reload: u8,
}

impl TriangleChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            length_counter: 0,
            timer: 0,
            timer_reload: 0,
            linear_counter: 0,
            linear_counter_reload: 0,
        }
    }

    pub fn write_register(&mut self, addr: u16, data: u8) {
        match addr {
            0x4008 => {
                self.linear_counter_reload = data & 0b0111_1111;
            }
            0x400A => {
                self.timer_reload = (self.timer_reload & 0xFF00) | data as u16;
            }
            0x400B => {
                self.timer_reload = (self.timer_reload & 0x00FF) | ((data as u16 & 0b111) << 8);
                self.length_counter = (data >> 3) & 0x1F;
            }
            _ => {}
        }
    }

    pub fn tick_sequencer(&mut self) {}

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
        }
    }

    pub fn decrement_length_counter(&mut self) {
        if self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn get_length_counter(&self) -> u8 {
        self.length_counter
    }

    pub fn generate_sample(&self) -> f32 {
        if !self.enabled || self.length_counter == 0 || self.linear_counter == 0 {
            return 0.0;
        }

        const TRIANGLE_WAVE: [f32; 32] = [
            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
            15.0, 14.0, 13.0, 12.0, 11.0, 10.0, 9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0, 0.0,
        ];

        let step = (self.timer % 32) as usize;

        TRIANGLE_WAVE[step] / 15.0
    }
}
