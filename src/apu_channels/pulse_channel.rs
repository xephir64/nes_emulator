use super::{envelope::Envelope, length_counter::LengthCounter, sweep_unit::SweepUnit};

const EIGHTH_DUTY_CYCLE: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 1];
const QUARTER_DUTY_CYCLE: [u8; 8] = [0, 0, 0, 0, 0, 0, 1, 1];
const HALF_DUTY_CYCLE: [u8; 8] = [0, 0, 0, 0, 1, 1, 1, 1];
const NEGATIVE_QUARTER_DUTY_CYCLE: [u8; 8] = [1, 1, 1, 1, 1, 1, 0, 0];
pub struct PulseChannel {
    enabled: bool,

    length_counter: LengthCounter,
    envelope: Envelope,
    sweep_unit: SweepUnit,

    duty_cycle: [u8; 8],

    sequence: usize,
    timer_load: u16,
    timer: u16,
}

impl PulseChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            length_counter: LengthCounter::new(),
            envelope: Envelope::new(),
            sweep_unit: SweepUnit::new(),
            duty_cycle: EIGHTH_DUTY_CYCLE,
            sequence: 0,
            timer_load: 0,
            timer: 0,
        }
    }

    pub fn write_register(&mut self, addr: u16, data: u8) {
        match addr % 4 {
            0 => {
                self.duty_cycle = match data >> 6 {
                    0b00 => EIGHTH_DUTY_CYCLE,
                    0b01 => QUARTER_DUTY_CYCLE,
                    0b10 => HALF_DUTY_CYCLE,
                    0b11 => NEGATIVE_QUARTER_DUTY_CYCLE,
                    _ => panic!(),
                };
                self.envelope.write_envelope(data);
                self.length_counter.set_halt(data & 0b0010_0000 != 0);
            }
            1 => self.sweep_unit.update(data),
            2 => {
                self.timer_load = (self.timer_load & 0b0111_0000_0000) | data as u16;
            }
            3 => {
                if self.enabled {
                    self.length_counter.set(data);
                }
                self.timer_load = (self.timer_load & 0b1111_1111) | ((data as u16 & 0b111) << 8);

                self.sequence = 0;
                self.envelope.set_start_flag();
            }
            _ => {}
        }
    }

    pub fn generate_sample(&self) -> u8 {
        if self.duty_cycle[self.sequence] != 0
            && self.length_counter.is_non_zero()
            && self.timer >= 8
        {
            self.envelope.volume()
        } else {
            0
        }
    }

    pub fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_load;

            self.sequence = (self.sequence + 1) & 7;
            println!(
                "Clocking PulseChannel, duty_cycle: {:?} to step {}",
                self.duty_cycle, self.sequence
            )
        } else {
            self.timer -= 1;
        }
    }

    pub fn set_enabled(&mut self, value: bool) {
        self.enabled = value;
        if !value {
            self.length_counter.disable();
        }
    }
}
