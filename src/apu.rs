use crate::apu_channels::{
    frame_counter::FrameCounter, pulse_channel::PulseChannel, triangle_channel::TriangleChannel,
};

pub struct Apu {
    pulse1: PulseChannel,
    pulse2: PulseChannel,
    triangle: TriangleChannel,
    frame_counter: FrameCounter,

    buffer: Vec<i16>,
}

impl Apu {
    pub fn new() -> Self {
        let clock_rate = 1789773; // NTSC Clock Rate
        let sample_rate = 44100;
        Self {
            pulse1: PulseChannel::new(),
            pulse2: PulseChannel::new(),
            triangle: TriangleChannel::new(),
            frame_counter: FrameCounter::new(),
            buffer: Vec::new(),
        }
    }

    fn generate_sample(&self) -> f32 {
        let pulse_out = self.pulse1.generate_sample() + self.pulse2.generate_sample();
        let triangle_out = self.triangle.generate_sample();

        //(pulse_out * 0.5 + triangle_out * 0.5) / 2.0
        0.0
    }

    pub fn tick(&mut self, cycles: usize) {
        self.triangle.tick_sequencer();

        if cycles % 2 == 1 {
            //self.pulse1.tick();
        }
    }

    pub fn take_samples(&mut self) -> Vec<i16> {
        let samples = self.buffer.clone();
        self.buffer.clear();
        samples
    }

    pub fn irq_pending(&mut self) -> bool {
        !self.frame_counter.is_interrupt_inhibit()
    }

    pub fn write_register(&mut self, addr: u16, data: u8, cycles: usize) {
        match addr {
            0x4000..=0x4003 => self.pulse1.write_register(addr, data),
            0x4004..=0x4007 => self.pulse2.write_register(addr, data),
            0x4008..=0x400B => self.triangle.write_register(addr, data),
            0x4015 => {
                self.pulse1.set_enabled(data & 0b0000_0001 != 0);
                self.pulse2.set_enabled(data & 0b0000_0010 != 0);
                self.triangle.set_enabled(data & 0b0000_0100 != 0);
            }
            0x4017 => self.frame_counter.write_control(data),
            _ => {}
        }
    }

    pub fn read_register(&self) -> u8 {
        0
    }
}
