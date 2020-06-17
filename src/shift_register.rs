#[derive(Default)]
pub struct ShiftRegister {
    shift_amount: u8,
    shift_data: u16,
}

impl ShiftRegister {
    pub fn new() -> Self {
        ShiftRegister {
            shift_amount: 0,
            shift_data: 0
        }
    }

    pub fn set_shift_amount(&mut self, val: u8) {
        self.shift_amount = val & 0b111;
    }

    pub fn put_value(&mut self, val: u8) {
        self.shift_data >>= 8;
        self.shift_data |= ((val as u16) << 8);
    }

    pub fn read_value(&self) -> u8 {
        let bits_to_shift = (8 - self.shift_amount) as u16;
        return ((self.shift_data >> bits_to_shift) & 0xFF) as u8;
    }
}
