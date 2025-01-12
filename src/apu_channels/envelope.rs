pub struct Envelope {
    constant_volume: u8,
    loop_envelope: bool,
    use_envelope: bool,
    start_flag: bool,
    decay_level: u8,
    divider: u8,
}

impl Envelope {
    pub fn new() -> Self {
        Envelope {
            constant_volume: 0,
            loop_envelope: false,
            use_envelope: false,
            start_flag: false,
            decay_level: 0,
            divider: 0,
        }
    }

    pub fn clock(&mut self) {
        if self.start_flag {
            self.start_flag = false;
            self.decay_level = 15;
            self.divider = self.constant_volume;
        } else if self.divider == 0 {
            self.divider = self.constant_volume;
            if self.decay_level == 0 {
                if self.loop_envelope {
                    self.decay_level = 15;
                }
            } else {
                self.decay_level -= 1;
            }
        } else {
            self.divider -= 1;
        }
    }

    pub fn write_envelope(&mut self, value: u8) {
        self.loop_envelope = value & 0b0010_0000 != 0;
        self.use_envelope = value & 0b0001_0000 != 0;
        self.constant_volume = value & 0xF;
    }

    pub fn volume(&self) -> u8 {
        if self.use_envelope {
            self.decay_level
        } else {
            self.constant_volume
        }
    }

    pub fn set_start_flag(&mut self) {
        self.start_flag = true;
    }
}
