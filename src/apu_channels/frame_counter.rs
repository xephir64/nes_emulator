use super::{pulse_channel::PulseChannel, triangle_channel::TriangleChannel};

pub struct FrameCounter {
    step: u8,
}

impl FrameCounter {
    pub fn new() -> Self {
        Self { step: 0 }
    }

    pub fn tick(
        &mut self,
        pulse1: &mut PulseChannel,
        pulse2: &mut PulseChannel,
        triangle: &mut TriangleChannel,
    ) {
        self.step = (self.step + 1) % 4;

        if self.step % 2 == 0 {
            pulse1.decrement_length_counter();
            pulse2.decrement_length_counter();
            triangle.decrement_length_counter();
        }
    }
}
