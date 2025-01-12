pub enum InterruptType {
    NMI,
    IRQ,
    BRK,
    RST,
}

pub struct Interrupt {
    pub int_type: InterruptType,
    pub vector_addr: u16,
    pub b_flag_mask: u8,
    pub cpu_cycles: u8,
}

pub const NMI: Interrupt = Interrupt {
    int_type: InterruptType::NMI,
    vector_addr: 0xFFFA,
    b_flag_mask: 0b0010_0000,
    cpu_cycles: 2,
};

pub const IRQ: Interrupt = Interrupt {
    int_type: InterruptType::IRQ,
    vector_addr: 0xFFFE,
    b_flag_mask: 0b0010_0000,
    cpu_cycles: 2,
};

pub const BRK: Interrupt = Interrupt {
    int_type: InterruptType::BRK,
    vector_addr: 0xFFFE,
    b_flag_mask: 0b0011_0000,
    cpu_cycles: 1,
};

pub const RST: Interrupt = Interrupt {
    int_type: InterruptType::RST,
    vector_addr: 0xFFFC,
    b_flag_mask: 0,
    cpu_cycles: 0,
};
