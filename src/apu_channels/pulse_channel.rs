pub struct PulseChannel {
    enabled: bool,
    duty: u8,
    length_counter: u8,
    timer: u16,
    timer_reload: u16,
    current_step: u8,
}

impl PulseChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            duty: 0,
            length_counter: 0,
            timer: 0,
            timer_reload: 0,
            current_step: 0,
        }
    }

    pub fn write_register(&mut self, addr: u16, data: u8) {
        match addr {
            0x4000 | 0x4004 => {
                self.duty = (data >> 6) & 0b11;
                self.length_counter = data & 0b0011_1111;
            }
            0x4002 | 0x4006 => {
                self.timer_reload = (self.timer_reload & 0xFF00) | data as u16;
            }
            0x4003 | 0x4007 => {
                self.timer_reload = (self.timer_reload & 0x00FF) | ((data as u16 & 0b111) << 8);
                self.length_counter = (data >> 3) & 0x1F;
            }
            _ => {}
        }
    }

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
        if !self.enabled || self.length_counter == 0 {
            return 0.0;
        }

        let duty_patterns = [
            0b00000001, // 12.5%
            0b00000011, // 25%
            0b00001111, // 50%
            0b11111100, // 75%
        ];
        let duty_pattern = duty_patterns[self.duty as usize];
        let step = (self.timer % 8) as u8;

        if (duty_pattern >> (7 - step)) & 1 == 0 {
            0.0
        } else {
            0.8
        }
    }
}
