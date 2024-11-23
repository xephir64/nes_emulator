use crate::apu_channels::{
    frame_counter::FrameCounter, pulse_channel::PulseChannel, triangle_channel::TriangleChannel,
};

pub struct Apu {
    pulse1: PulseChannel,
    pulse2: PulseChannel,
    triangle: TriangleChannel,
    frame_counter: FrameCounter,

    sample_buffer: Vec<f32>,
    sample_rate: u32,
    sample_timer: f64,
    clock_rate: u32,
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
            sample_buffer: Vec::new(),
            sample_rate,
            sample_timer: 0.0,
            clock_rate,
        }
    }

    fn generate_sample(&self) -> f32 {
        let pulse_out = self.pulse1.generate_sample() + self.pulse2.generate_sample();
        let triangle_out = self.triangle.generate_sample();

        (pulse_out * 0.5 + triangle_out * 0.5) / 2.0
    }

    pub fn tick(&mut self) {
        self.frame_counter
            .tick(&mut self.pulse1, &mut self.pulse2, &mut self.triangle);

        self.sample_timer += (self.clock_rate / self.sample_rate) as f64;

        if self.sample_timer >= 1.0 {
            self.sample_timer -= 1.0;
            let sample = self.generate_sample();
            self.sample_buffer.push(sample);
        }
    }

    pub fn take_samples(&mut self) -> Vec<f32> {
        let samples = self.sample_buffer.clone();
        self.sample_buffer.clear();
        samples
    }

    pub fn write_register(&mut self, addr: u16, data: u8) {
        println!("APU write at {:x}", addr);
        match addr {
            0x4000..=0x4003 => self.pulse1.write_register(addr, data),
            0x4004..=0x4007 => self.pulse2.write_register(addr, data),
            0x4008..=0x400B => self.triangle.write_register(addr, data),
            0x4015 => {
                self.pulse1.set_enabled(data & 0b0000_0001 != 0);
                self.pulse2.set_enabled(data & 0b0000_0010 != 0);
                self.triangle.set_enabled(data & 0b0000_0100 != 0);
            }
            _ => {}
        }
    }

    pub fn read_register(&self, addr: u16) -> u8 {
        print!("APU read at {:x}", addr);
        match addr {
            0x4000..=0x4007 => {
                println!(
                    "Warning: Read from write-only pulse/triangle register at {:04X}",
                    addr
                );
                0
            }
            0x4008..=0x400B => {
                println!(
                    "Warning: Read from write-only triangle register at {:04X}",
                    addr
                );
                0
            }
            0x4015 => {
                let mut status = 0;
                if self.pulse1.get_length_counter() > 0 {
                    status |= 0b0000_0001;
                }
                if self.pulse2.get_length_counter() > 0 {
                    status |= 0b0000_0010;
                }
                if self.triangle.get_length_counter() > 0 {
                    status |= 0b0000_0100;
                }
                status
            }
            _ => {
                println!("Warning: Invalid APU read at {:04X}", addr);
                0
            }
        }
    }
}
