use super::{pulse_channel::PulseChannel, triangle_channel::TriangleChannel};
use bitflags::bitflags;

bitflags! {
    pub struct FrameCounterFlags: u8 {
        const SEQUENCER_MODE = 0b1000_0000; // Bit 7
        const INTERRUPT_INHIBIT = 0b0100_0000; // Bit 6
    }
}

pub struct FrameCounter {
    step: u8,
    mode: FrameCounterFlags,
}

impl FrameCounter {
    pub fn new() -> Self {
        Self {
            step: 0,
            mode: FrameCounterFlags::empty(),
        }
    }

    pub fn write_control(&mut self, value: u8) {
        self.mode = FrameCounterFlags::from_bits_truncate(value);
        if self.mode.contains(FrameCounterFlags::INTERRUPT_INHIBIT) {
            self.clear_frame_interrupt_flag();
        }
    }

    pub fn tick(
        &mut self,
        pulse1: &mut PulseChannel,
        pulse2: &mut PulseChannel,
        triangle: &mut TriangleChannel,
    ) {
        let max_steps = if self.mode.contains(FrameCounterFlags::SEQUENCER_MODE) {
            5
        } else {
            4
        };

        self.step = (self.step + 1) % max_steps;

        match (
            self.mode.contains(FrameCounterFlags::SEQUENCER_MODE),
            self.step,
        ) {
            (false, 0 | 2) | (true, 1 | 3 | 4) => {
                /*pulse1.decrement_length_counter();
                pulse2.decrement_length_counter();*/
                triangle.decrement_length_counter();
            }
            _ => {}
        }

        if !self.mode.contains(FrameCounterFlags::INTERRUPT_INHIBIT) {
            self.trigger_frame_interrupt();
        }
    }

    fn clear_frame_interrupt_flag(&mut self) {
        self.mode.remove(FrameCounterFlags::INTERRUPT_INHIBIT);
    }

    fn trigger_frame_interrupt(&mut self) {
        self.mode.set(FrameCounterFlags::INTERRUPT_INHIBIT, true);
    }

    pub fn is_interrupt_inhibit(&mut self) -> bool {
        self.mode.contains(FrameCounterFlags::INTERRUPT_INHIBIT)
    }
}
