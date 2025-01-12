pub struct SweepUnit {
    enabled: bool,
    divider_period: u8,
    is_negate: bool,
    shift_count: u8,
}

impl SweepUnit {
    pub fn new() -> Self {
        SweepUnit {
            enabled: false,
            divider_period: 0,
            is_negate: false,
            shift_count: 0,
        }
    }

    pub fn update(&mut self, value: u8) {
        self.enabled = value & 0b1000_0000 == 0b1000_0000;
        self.divider_period = (value & 0b0111_0000) >> 4;
        self.is_negate = value & 0b0000_1000 == 0b0000_1000;
        self.shift_count = value & 0b0000_0111;
    }
}
