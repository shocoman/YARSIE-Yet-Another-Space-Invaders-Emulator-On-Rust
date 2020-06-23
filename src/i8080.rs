const INSTR_CYCLES: [u8; 0x100] = [
    4, 10, 7,  5, 5,  5,  7,  4, 4, 10, 7,  5, 5, 5, 7, 4, //0x0...
    4, 10, 7,  5, 5,  5,  7,  4, 4, 10, 7,  5, 5, 5, 7, 4, //0x1...
    4, 10, 16, 5, 5,  5,  7,  4, 4, 10, 16, 5, 5, 5, 7, 4, //0x2...
    4, 10, 13, 5, 10, 10, 10, 4, 4, 10, 13, 5, 5, 5, 7, 4, //0x3...

    5, 5, 5, 5, 5, 5, 7, 5, 5, 5, 5, 5, 5, 5, 7, 5, //0x4...
    5, 5, 5, 5, 5, 5, 7, 5, 5, 5, 5, 5, 5, 5, 7, 5, //0x5...
    5, 5, 5, 5, 5, 5, 7, 5, 5, 5, 5, 5, 5, 5, 7, 5, //0x6...
    7, 7, 7, 7, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 7, 5, //0x7...

    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, //0x8...
    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, //0x9...
    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, //0xA...
    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, //0xB...

    11, 10, 10, 10, 17, 11, 7, 11, 11, 10, 10, 10, 10, 17, 7, 11, //0xC...
    11, 10, 10, 10, 17, 11, 7, 11, 11, 10, 10, 10, 10, 17, 7, 11, //0xD...
    11, 10, 10, 18, 17, 11, 7, 11, 11, 5,  10, 5,  17, 17, 7, 11, //0xE...
    11, 10, 10, 4,  17, 11, 7, 11, 11, 5,  10, 4,  17, 17, 7, 11, //0xF...
];


#[derive(Clone, Copy)]
enum FlagBit {
    Carry = 0,
    Parity = 2,
    AuxiliaryCarry = 4,
    Zero = 6,
    Sign = 7,
}

pub struct I8080 {
    pub memory: [u8; 0x10000],

    pub a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,

    flags: u8, // S, Z, -, AC, -, P, -, C
    pub sp: usize,
    pub pc: usize,

    pub enable_interrupts: bool,
    pub halted: bool,
}

impl I8080 {

    pub fn new() -> Self {
        I8080 {
            memory: [0; 0x10000],
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,

            flags: 0,
            sp: 0xFFFF,
            pc: 0,
            enable_interrupts: false,
            halted: false,
        }
    }

    pub fn set_zsp_flags(&mut self, val: u8) {
        self.set_flag_bit(FlagBit::Zero, val == 0);
        self.set_flag_bit(FlagBit::Sign, (val & (0x1 << 7)) != 0);
        self.set_flag_bit(FlagBit::Parity, val.count_ones() % 2 == 0);
    }

    fn set_flag_bit(&mut self, bit: FlagBit, state: bool) {
        let bit_num = bit as u8;
        self.flags &= !(0x1 << bit_num);
        self.flags |= (state as u8) << bit_num;
    }

    fn get_flag_bit(&self, bit: FlagBit) -> bool { self.flags & (0x1 << bit as u8) != 0 }

    pub fn join_bytes(high: u8, low: u8) -> usize {
        (high as usize) << 8 | low as usize
    }

    pub fn print_state(&self) {
        println!(" --- \ni8080 state: ");
        println!(
            "\tA: {:x}, B: {:x}, C: {:x}, D: {:x}, E: {:x}, H: {:x}, L: {:x}",
            self.a, self.b, self.c, self.d, self.e, self.h, self.l
        );

        println!("\tPC: {:x}, SP: {:x}, EI: {}; INSTR: {:x}\n\tFlags = Zero: {}, Sign: {}, Parity: {}, Carry: {}, AuxCarry: {}",
            self.pc, self.sp, self.enable_interrupts, self.read_memory(self.pc),
            self.get_flag_bit(FlagBit::Zero) as u8, self.get_flag_bit(FlagBit::Sign)  as u8,
            self.get_flag_bit(FlagBit::Parity) as u8, self.get_flag_bit(FlagBit::Carry) as u8,
            self.get_flag_bit(FlagBit::AuxiliaryCarry) as u8);
    }

    pub fn load_rom(&mut self, rom: &Vec<u8>, offset: usize){
        for (byte_num, byte) in rom.iter().enumerate() {
            self.memory[offset + byte_num] = *byte;
        }
    }

    pub fn read_instr(&self) -> Option<u8> {
        self.memory.get(self.pc).copied()
    }

    pub fn generate_interrupt(&mut self, int_num: usize) {
        if self.enable_interrupts {
            self.push(self.pc);
            self.pc = 0x8 * int_num;
            self.enable_interrupts = false;
        }
    }

    pub fn push(&mut self, data: usize) {
        self.write_memory(self.sp - 2, (data & 0xFF) as u8);
        self.write_memory(self.sp - 1, ((data >> 8) & 0xFF) as u8);
        self.sp -= 2;
    }

    pub fn pop(&mut self) -> usize {
        self.sp += 2;
        Self::join_bytes(self.read_memory(self.sp - 1), self.read_memory(self.sp - 2))
    }

    fn add_with_carry(&mut self, addend: u8){
        let carry_bit = self.get_flag_bit(FlagBit::Carry) as u8;
        let (add_data, is_overflow1) = self.a.overflowing_add(addend);
        let (add_one, is_overflow2) = add_data.overflowing_add(self.get_flag_bit(FlagBit::Carry) as u8);
        self.set_zsp_flags(add_one);
        self.set_flag_bit(FlagBit::Carry, is_overflow1 || is_overflow2);
        self.set_flag_bit(FlagBit::AuxiliaryCarry, (self.a & 0xF) + (addend & 0xF) + carry_bit > 0xF);
        self.a = add_one;
    }

    fn sub_with_borrow(&mut self, subtrahend: u8){
        let carry_bit = self.get_flag_bit(FlagBit::Carry) as u8;
        let (sub_data, is_overflow1) = self.a.overflowing_sub(subtrahend);
        let (sub_one, is_overflow2) = sub_data.overflowing_sub(carry_bit);
        self.set_zsp_flags(sub_one);
        self.set_flag_bit(FlagBit::Carry, is_overflow1 || is_overflow2);
        self.set_flag_bit(FlagBit::AuxiliaryCarry, self.a & 0xF < (subtrahend & 0xF) + carry_bit);
        self.a = sub_one;
    }

    fn add(&mut self, addend: u8){
        let (add_data, is_overflow) = self.a.overflowing_add(addend);
        self.set_zsp_flags(add_data);
        self.set_flag_bit(FlagBit::Carry, is_overflow);
        self.set_flag_bit(FlagBit::AuxiliaryCarry, (self.a & 0xF) + (addend & 0xF) > 0xF);
        self.a = add_data;
    }

    fn sub(&mut self, subtrahend: u8){
        let (sub_data, is_overflow) = self.a.overflowing_sub(subtrahend);
        self.set_zsp_flags(sub_data);
        self.set_flag_bit(FlagBit::Carry, is_overflow);
        self.set_flag_bit(FlagBit::AuxiliaryCarry, self.a & 0xF < subtrahend & 0xF);
        self.a = sub_data;
    }

    fn and_logical(&mut self, rhs: u8){
        self.a &= rhs;
        self.set_zsp_flags(self.a);
        self.set_flag_bit(FlagBit::Carry, false);
        self.set_flag_bit(FlagBit::AuxiliaryCarry, false);
    }

    fn xor_logical(&mut self, rhs: u8){
        self.a ^= rhs;
        self.set_zsp_flags(self.a);
        self.set_flag_bit(FlagBit::Carry, false);
        self.set_flag_bit(FlagBit::AuxiliaryCarry, false);
    }

    fn or_logical(&mut self, rhs: u8){
        self.a |= rhs;
        self.set_zsp_flags(self.a);
        self.set_flag_bit(FlagBit::Carry, false);
        self.set_flag_bit(FlagBit::AuxiliaryCarry, false);
    }

    fn compare(&mut self, rhs: u8){
        let (result, is_overflow) = self.a.overflowing_sub(rhs);
        self.set_zsp_flags(result);
        self.set_flag_bit(FlagBit::Carry, is_overflow);
        self.set_flag_bit(FlagBit::AuxiliaryCarry, self.a & 0xF < rhs & 0xF);
    }

    pub fn read_memory(&self, address: usize) -> u8 {
        self.memory[address]
    }

    pub fn write_memory(&mut self, address: usize, value: u8){
        self.memory[address] = value;
    }

    pub fn execute(&mut self, instr: u8) -> usize {
        match instr {
            0x00 => {
                self.pc += 1;
            } // NOP
            0x01 => {
                self.b = self.read_memory(self.pc + 2);
                self.c = self.read_memory(self.pc + 1);
                self.pc += 3;
            } // LXI B,D16
            0x02 => {
                let mem_addr = Self::join_bytes(self.b, self.c);
                self.write_memory(mem_addr, self.a);
                self.pc += 1;
            } // STAX B
            0x03 => {
                let bc = (Self::join_bytes(self.b, self.c) as u16).wrapping_add(1);
                self.b = (bc >> 8 & 0xFF) as u8;
                self.c = (bc & 0xFF) as u8;
                self.pc += 1;
            } // INX B
            0x04 => {
                self.b = self.b.wrapping_add(1);
                self.set_zsp_flags(self.b);
                self.set_flag_bit(FlagBit::AuxiliaryCarry, self.b == 0x10);
                self.pc += 1;
            } // INR B
            0x05 => {
                self.b = self.b.wrapping_sub(1);
                self.set_zsp_flags(self.b);
                self.set_flag_bit(FlagBit::AuxiliaryCarry, self.b & 0xF == 0xF);
                self.pc += 1;
            } // DCR B
            0x06 => {
                self.b = self.read_memory(self.pc + 1);
                self.pc += 2;
            } // MVI B, D8
            0x07 => {
                self.set_flag_bit(FlagBit::Carry, self.a >> 7 & 0x1 != 0);
                self.a = self.a.rotate_left(1);
                self.pc += 1;
            } // RLC
            0x09 => {
                let prev_hl = Self::join_bytes(self.h, self.l) as u16;
                let bc = Self::join_bytes(self.b, self.c) as u16;
                let (hl, is_overflow) = prev_hl.overflowing_add(bc);

                self.h = (hl >> 8 & 0xFF) as u8;
                self.l = (hl & 0xFF) as u8;
                self.set_flag_bit(FlagBit::Carry, is_overflow);
                self.pc += 1;
            } // DAD B
            0x0a => {
                let mem_addr = Self::join_bytes(self.b, self.c);
                self.a = self.read_memory(mem_addr);
                self.pc += 1;
            } // LDAX B
            0x0b => {
                let bc = (Self::join_bytes(self.b, self.c) as u16).wrapping_sub(1);
                self.b = (bc >> 8 & 0xFF) as u8;
                self.c = (bc & 0xFF) as u8;
                self.pc += 1;
            } // DCX B
            0x0c => {
                self.c = self.c.wrapping_add(1);
                self.set_zsp_flags(self.c);
                self.set_flag_bit(FlagBit::AuxiliaryCarry, self.c == 0x10);
                self.pc += 1;
            } // INR C
            0x0d => {
                self.c = self.c.wrapping_sub(1);
                self.set_zsp_flags(self.c);
                self.set_flag_bit(FlagBit::AuxiliaryCarry, self.c & 0xF == 0xF);
                self.pc += 1;
            } // DCR C
            0x0e => {
                self.c = self.read_memory(self.pc + 1);
                self.pc += 2;
            } // MVI C,D8
            0x0f => {
                self.set_flag_bit(FlagBit::Carry, self.a & 0x1 != 0);
                self.a = self.a.rotate_right(1);
                self.pc += 1;
            } // RRC
            0x11 => {
                self.d = self.read_memory(self.pc + 2);
                self.e = self.read_memory(self.pc + 1);
                self.pc += 3;
            } // LXI D,D16
            0x12 => {
                let mem_addr = Self::join_bytes(self.d, self.e);
                self.write_memory(mem_addr, self.a);
                self.pc += 1;
            } // STAX D
            0x13 => {
                let de = (Self::join_bytes(self.d, self.e) as u16).wrapping_add(1);
                self.d = (de >> 8 & 0xFF) as u8;
                self.e = (de & 0xFF) as u8;
                self.pc += 1;
            } // INX D
            0x14 => {
                self.d = self.d.wrapping_add(1);
                self.set_zsp_flags(self.d);
                self.set_flag_bit(FlagBit::AuxiliaryCarry, self.d == 0x10);
                self.pc += 1;
            } // INR D
            0x15 => {
                self.d = self.d.wrapping_sub(1);
                self.set_zsp_flags(self.d);
                self.set_flag_bit(FlagBit::AuxiliaryCarry, self.d & 0xF == 0xF);
                self.pc += 1;
            } // DCR D
            0x16 => {
                self.d = self.read_memory(self.pc + 1);
                self.pc += 2;
            } // MVI D, D8
            0x17 => {
                let carry_bit = self.get_flag_bit(FlagBit::Carry) as u8;
                self.set_flag_bit(FlagBit::Carry, (self.a >> 7) & 0x1 != 0);
                self.a = self.a.rotate_left(1) & !0x1 | carry_bit;
                self.pc += 1;
            } // RAL
            0x19 => {
                let prev_hl = Self::join_bytes(self.h, self.l) as u16;
                let de = Self::join_bytes(self.d, self.e) as u16;
                let (hl, is_overflow) = prev_hl.overflowing_add(de);

                self.h = (hl >> 8 & 0xFF) as u8;
                self.l = (hl & 0xFF) as u8;
                self.set_flag_bit(FlagBit::Carry, is_overflow);
                self.pc += 1;
            } // DAD D
            0x1a => {
                let mem_addr = Self::join_bytes(self.d, self.e);
                self.a = self.read_memory(mem_addr);
                self.pc += 1;
            } // LDAX D
            0x1b => {
                let de = (Self::join_bytes(self.d, self.e) as u16).wrapping_sub(1);
                self.d = (de >> 8 & 0xFF) as u8;
                self.e = (de & 0xFF) as u8;
                self.pc += 1;
            } // DCX D
            0x1c => {
                self.e = self.e.wrapping_add(1);
                self.set_zsp_flags(self.e);
                self.set_flag_bit(FlagBit::AuxiliaryCarry, self.e == 0x10);
                self.pc += 1;
            } // INR E
            0x1d => {
                self.e = self.e.wrapping_sub(1);
                self.set_zsp_flags(self.e);
                self.set_flag_bit(FlagBit::AuxiliaryCarry, self.e & 0xF == 0xF);
                self.pc += 1;
            } // DCR E
            0x1e => {
                self.e = self.read_memory(self.pc + 1);
                self.pc += 2;
            } // MVI E,D8
            0x1f => {
                let carry_bit = self.get_flag_bit(FlagBit::Carry) as u8;
                self.set_flag_bit(FlagBit::Carry, self.a & 0x1 != 0);
                self.a = self.a.rotate_right(1) & !(0x1 << 7) | (carry_bit << 7);
                self.pc += 1;
            } // RAR
            0x20 => {
                self.pc += 1;
            } // RIM // NOPE
            0x21 => {
                self.h = self.read_memory(self.pc + 2);
                self.l = self.read_memory(self.pc + 1);
                self.pc += 3;
            } // LXI H,D16
            0x22 => {
                let mem_addr = Self::join_bytes(self.read_memory(self.pc + 2), self.read_memory(self.pc + 1));
                self.write_memory(mem_addr, self.l);
                self.write_memory(mem_addr + 1, self.h);
                self.pc += 3;
            } // SHLD adr
            0x23 => {
                let hl = (Self::join_bytes(self.h, self.l) as u16).wrapping_add(1);
                self.h = (hl >> 8 & 0xFF) as u8;
                self.l = (hl & 0xFF) as u8;
                self.pc += 1;
            } // INX H
            0x24 => {
                self.h = self.h.wrapping_add(1);
                self.set_zsp_flags(self.h);
                self.set_flag_bit(FlagBit::AuxiliaryCarry, self.h == 0x10);
                self.pc += 1;
            } // INR H
            0x25 => {
                self.h = self.h.wrapping_sub(1);
                self.set_zsp_flags(self.h);
                self.set_flag_bit(FlagBit::AuxiliaryCarry, self.h & 0xF == 0xF);
                self.pc += 1;
            } // DCR H
            0x26 => {
                self.h = self.read_memory(self.pc + 1);
                self.pc += 2;
            } // MVI H,D8
            0x27 => {
                if self.a & 0xF > 0x9 || self.get_flag_bit(FlagBit::AuxiliaryCarry) {
                    self.set_flag_bit(FlagBit::AuxiliaryCarry, true);
                    self.set_flag_bit(FlagBit::Carry, (self.a >> 4) & 0xF > 0x9 || self.get_flag_bit(FlagBit::Carry));
                    self.a = self.a.wrapping_add(0x6);
                }
                if (self.a >> 4) & 0xF > 0x9 || self.get_flag_bit(FlagBit::Carry) {
                    self.set_flag_bit(FlagBit::Carry, true);
                    self.a = self.a.wrapping_add(0x60);
                }
                self.set_zsp_flags(self.a);
                self.pc += 1;
            } // DAA // DECIMAL ADJUST ACCUMULATOR
            0x29 => {
                let prev_hl = Self::join_bytes(self.h, self.l) as u16;
                let (hl, is_overflow) = prev_hl.overflowing_add(prev_hl);

                self.h = (hl >> 8 & 0xFF) as u8;
                self.l = (hl & 0xFF) as u8;
                self.set_flag_bit(FlagBit::Carry, is_overflow);
                self.pc += 1;
            } // DAD H
            0x2a => {
                let mem_addr = Self::join_bytes(self.read_memory(self.pc + 2), self.read_memory(self.pc + 1));
                self.l = self.read_memory(mem_addr);
                self.h = self.read_memory(mem_addr + 1);
                self.pc += 3;
            } // LHLD adr
            0x2b => {
                let hl = (Self::join_bytes(self.h, self.l) as u16).wrapping_sub(1);
                self.h = (hl >> 8 & 0xFF) as u8;
                self.l = (hl & 0xFF) as u8;
                self.pc += 1;
            } // DCX H
            0x2c => {
                self.l = self.l.wrapping_add(1);
                self.set_zsp_flags(self.l);
                self.set_flag_bit(FlagBit::AuxiliaryCarry, self.l == 0x10);
                self.pc += 1;
            } // INR L
            0x2d => {
                self.l = self.l.wrapping_sub(1);
                self.set_zsp_flags(self.l);
                self.set_flag_bit(FlagBit::AuxiliaryCarry, self.l & 0xF == 0xF);
                self.pc += 1;
            } // DCR L
            0x2e => {
                self.l = self.read_memory(self.pc + 1);
                self.pc += 2;
            } // MVI L, D8
            0x2f => {
                self.a = !self.a;
                self.pc += 1;
            } // CMA
            0x30 => {
                self.pc += 1;
            } // SIM // NOPE
            0x31 => {
                self.sp = Self::join_bytes(self.read_memory(self.pc + 2), self.read_memory(self.pc + 1));
                self.pc += 3;
            } // LXI SP, D16
            0x32 => {
                let mem_addr = Self::join_bytes(self.read_memory(self.pc + 2), self.read_memory(self.pc + 1));
                self.write_memory(mem_addr, self.a);
                self.pc += 3;
            } // STA adr
            0x33 => {
                self.sp = (self.sp as u16).wrapping_add(1) as usize;
                self.pc += 1;
            } // INX SP
            0x34 => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.write_memory(mem_addr, self.read_memory(mem_addr).wrapping_add(1));
                self.set_zsp_flags(self.read_memory(mem_addr));
                self.set_flag_bit(FlagBit::AuxiliaryCarry, self.read_memory(mem_addr) == 0x10);
                self.pc += 1;
            } // INR M
            0x35 => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.write_memory(mem_addr, self.read_memory(mem_addr).wrapping_sub(1));
                self.set_zsp_flags(self.read_memory(mem_addr));
                self.set_flag_bit(FlagBit::AuxiliaryCarry, self.read_memory(mem_addr) & 0xF == 0xF);
                self.pc += 1;
            } // DCR M
            0x36 => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.write_memory(mem_addr, self.read_memory(self.pc + 1));
                self.pc += 2;
            } // MVI M,D8
            0x37 => {
                self.set_flag_bit(FlagBit::Carry, true);
                self.pc += 1;
            } // STC
            0x39 => {
                let prev_hl = Self::join_bytes(self.h, self.l) as u16;
                let (hl, is_overflow) = prev_hl.overflowing_add(self.sp as u16);

                self.h = (hl >> 8 & 0xFF) as u8;
                self.l = (hl & 0xFF) as u8;
                self.set_flag_bit(FlagBit::Carry, is_overflow);
                self.pc += 1;
            } // DAD SP
            0x3a => {
                let mem_addr = Self::join_bytes(self.read_memory(self.pc + 2), self.read_memory(self.pc + 1));
                self.a = self.read_memory(mem_addr);
                self.pc += 3;
            } // LDA adr
            0x3b => {
                self.sp = (self.sp as u16).wrapping_sub(1) as usize;
                self.pc += 1;
            } // DCX SP
            0x3c => {
                self.a = self.a.wrapping_add(1);
                self.set_zsp_flags(self.a);
                self.set_flag_bit(FlagBit::AuxiliaryCarry, self.a == 0x10);
                self.pc += 1;
            } // INR A
            0x3d => {
                self.a = self.a.wrapping_sub(1);
                self.set_zsp_flags(self.a);
                self.set_flag_bit(FlagBit::AuxiliaryCarry, self.a & 0xF == 0xF);
                self.pc += 1;
            } // DCR A
            0x3e => {
                self.a = self.read_memory(self.pc + 1);
                self.pc += 2;
            } // MVI A,D8
            0x3f => {
                self.set_flag_bit(FlagBit::Carry, !self.get_flag_bit(FlagBit::Carry));
                self.pc += 1;
            } // CMC
            0x40 => {
                self.b = self.b;
                self.pc += 1;
            } // MOV B,B
            0x41 => {
                self.b = self.c;
                self.pc += 1;
            } // MOV B,C
            0x42 => {
                self.b = self.d;
                self.pc += 1;
            } // MOV B,D
            0x43 => {
                self.b = self.e;
                self.pc += 1;
            } // MOV B,E
            0x44 => {
                self.b = self.h;
                self.pc += 1;
            } // MOV B,H
            0x45 => {
                self.b = self.l;
                self.pc += 1;
            } // MOV B,L
            0x46 => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.b = self.read_memory(mem_addr);
                self.pc += 1;
            } // MOV B,M
            0x47 => {
                self.b = self.a;
                self.pc += 1;
            } // MOV B,A
            0x48 => {
                self.c = self.b;
                self.pc += 1;
            } // MOV C,B
            0x49 => {
                self.c = self.c;
                self.pc += 1;
            } // MOV C,C
            0x4a => {
                self.c = self.d;
                self.pc += 1;
            } // MOV C,D
            0x4b => {
                self.c = self.e;
                self.pc += 1;
            } // MOV C,E
            0x4c => {
                self.c = self.h;
                self.pc += 1;
            } // MOV C,H
            0x4d => {
                self.c = self.l;
                self.pc += 1;
            } // MOV C,L
            0x4e => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.c = self.read_memory(mem_addr);
                self.pc += 1;
            } // MOV C,M
            0x4f => {
                self.c = self.a;
                self.pc += 1;
            } // MOV C,A
            0x50 => {
                self.d = self.b;
                self.pc += 1;
            } // MOV D,B
            0x51 => {
                self.d = self.c;
                self.pc += 1;
            } // MOV D,C
            0x52 => {
                self.d = self.d;
                self.pc += 1;
            } // MOV D,D
            0x53 => {
                self.d = self.e;
                self.pc += 1;
            } // MOV D,E
            0x54 => {
                self.d = self.h;
                self.pc += 1;
            } // MOV D,H
            0x55 => {
                self.d = self.l;
                self.pc += 1;
            } // MOV D,L
            0x56 => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.d = self.read_memory(mem_addr);
                self.pc += 1;
            } // MOV D,M
            0x57 => {
                self.d = self.a;
                self.pc += 1;
            } // MOV D,A
            0x58 => {
                self.e = self.b;
                self.pc += 1;
            } // MOV E,B
            0x59 => {
                self.e = self.c;
                self.pc += 1;
            } // MOV E,C
            0x5a => {
                self.e = self.d;
                self.pc += 1;
            } // MOV E,D
            0x5b => {
                self.e = self.e;
                self.pc += 1;
            } // MOV E,E
            0x5c => {
                self.e = self.h;
                self.pc += 1;
            } // MOV E,H
            0x5d => {
                self.e = self.l;
                self.pc += 1;
            } // MOV E,L
            0x5e => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.e = self.read_memory(mem_addr);
                self.pc += 1;
            } // MOV E,M
            0x5f => {
                self.e = self.a;
                self.pc += 1;
            } // MOV E,A
            0x60 => {
                self.h = self.b;
                self.pc += 1;
            } // MOV H,B
            0x61 => {
                self.h = self.c;
                self.pc += 1;
            } // MOV H,C
            0x62 => {
                self.h = self.d;
                self.pc += 1;
            } // MOV H,D
            0x63 => {
                self.h = self.e;
                self.pc += 1;
            } // MOV H,E
            0x64 => {
                self.h = self.h;
                self.pc += 1;
            } // MOV H,H
            0x65 => {
                self.h = self.l;
                self.pc += 1;
            } // MOV H,L
            0x66 => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.h = self.read_memory(mem_addr);
                self.pc += 1;
            } // MOV H,M
            0x67 => {
                self.h = self.a;
                self.pc += 1;
            } // MOV H,A
            0x68 => {
                self.l = self.b;
                self.pc += 1;
            } // MOV L,B
            0x69 => {
                self.l = self.c;
                self.pc += 1;
            } // MOV L,C
            0x6a => {
                self.l = self.d;
                self.pc += 1;
            } // MOV L,D
            0x6b => {
                self.l = self.e;
                self.pc += 1;
            } // MOV L,E
            0x6c => {
                self.l = self.h;
                self.pc += 1;
            } // MOV L,H
            0x6d => {
                self.l = self.l;
                self.pc += 1;
            } // MOV L,L
            0x6e => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.l = self.read_memory(mem_addr);
                self.pc += 1;
            } // MOV L,M
            0x6f => {
                self.l = self.a;
                self.pc += 1;
            } // MOV L,A
            0x70 => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.write_memory(mem_addr, self.b);
                self.pc += 1;
            } // MOV M,B
            0x71 => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.write_memory(mem_addr, self.c);
                self.pc += 1;
            } // MOV M,C
            0x72 => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.write_memory(mem_addr, self.d);
                self.pc += 1;
            } // MOV M,D
            0x73 => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.write_memory(mem_addr, self.e);
                self.pc += 1;
            } // MOV M,E
            0x74 => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.write_memory(mem_addr, self.h);
                self.pc += 1;
            } // MOV M,H
            0x75 => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.write_memory(mem_addr, self.l);
                self.pc += 1;
            } // MOV M,L
            0x76 => {
                self.halted = true;
                self.pc += 1;
            } // HLT
            0x77 => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.write_memory(mem_addr, self.a);
                self.pc += 1;
            } // MOV M,A
            0x78 => {
                self.a = self.b;
                self.pc += 1;
            } // MOV A,B
            0x79 => {
                self.a = self.c;
                self.pc += 1;
            } // MOV A,C
            0x7a => {
                self.a = self.d;
                self.pc += 1;
            } // MOV A,D
            0x7b => {
                self.a = self.e;
                self.pc += 1;
            } // MOV A,E
            0x7c => {
                self.a = self.h;
                self.pc += 1;
            } // MOV A,H
            0x7d => {
                self.a = self.l;
                self.pc += 1;
            } // MOV A,L
            0x7e => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.a = self.read_memory(mem_addr);
                self.pc += 1;
            } // MOV A,M
            0x7f => {
                self.a = self.a;
                self.pc += 1;
            } // MOV A,A
            0x80 => {
                self.add(self.b);
                self.pc += 1;
            } // ADD B
            0x81 => {
                self.add(self.c);
                self.pc += 1;
            } // ADD C
            0x82 => {
                self.add(self.d);
                self.pc += 1;
            } // ADD D
            0x83 => {
                self.add(self.e);
                self.pc += 1;
            } // ADD E
            0x84 => {
                self.add(self.h);
                self.pc += 1;
            } // ADD H
            0x85 => {
                self.add(self.l);
                self.pc += 1;
            } // ADD L
            0x86 => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.add(self.read_memory(mem_addr));
                self.pc += 1;
            } // ADD M
            0x87 => {
                self.add(self.a);
                self.pc += 1;
            } // ADD A
            0x88 => {
                self.add_with_carry(self.b);
                self.pc += 1;
            } // ADC B
            0x89 => {
                self.add_with_carry(self.c);
                self.pc += 1;
            } // ADC C
            0x8a => {
                self.add_with_carry(self.d);
                self.pc += 1;
            } // ADC D
            0x8b => {
                self.add_with_carry(self.e);
                self.pc += 1;
            } // ADC E
            0x8c => {
                self.add_with_carry(self.h);
                self.pc += 1;
            } // ADC H
            0x8d => {
                self.add_with_carry(self.l);
                self.pc += 1;
            } // ADC L
            0x8e => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.add_with_carry(self.read_memory(mem_addr));
                self.pc += 1;
            } // ADC M
            0x8f => {
                self.add_with_carry(self.a);
                self.pc += 1;
            } // ADC A
            0x90 => {
                self.sub(self.b);
                self.pc += 1;
            } // SUB B
            0x91 => {
                self.sub(self.c);
                self.pc += 1;
            } // SUB C
            0x92 => {
                self.sub(self.d);
                self.pc += 1;
            } // SUB D
            0x93 => {
                self.sub(self.e);
                self.pc += 1;
            } // SUB E
            0x94 => {
                self.sub(self.h);
                self.pc += 1;
            } // SUB H
            0x95 => {
                self.sub(self.l);
                self.pc += 1;
            } // SUB L
            0x96 => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.sub(self.read_memory(mem_addr));
                self.pc += 1;
            } // SUB M
            0x97 => {
                self.sub(self.a);
                self.pc += 1;
            } // SUB A
            0x98 => {
                self.sub_with_borrow(self.b);
                self.pc += 1;
            } // SBB B
            0x99 => {
                self.sub_with_borrow(self.c);
                self.pc += 1;
            } // SBB C
            0x9a => {
                self.sub_with_borrow(self.d);
                self.pc += 1;
            } // SBB D
            0x9b => {
                self.sub_with_borrow(self.e);
                self.pc += 1;
            } // SBB E
            0x9c => {
                self.sub_with_borrow(self.h);
                self.pc += 1;
            } // SBB H
            0x9d => {
                self.sub_with_borrow(self.l);
                self.pc += 1;
            } // SBB L
            0x9e => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.sub_with_borrow(self.read_memory(mem_addr));
                self.pc += 1;
            } // SBB M
            0x9f => {
                self.sub_with_borrow(self.a);
                self.pc += 1;
            } // SBB A
            0xa0 => {
                self.and_logical(self.b);
                self.pc += 1;
            } // ANA B
            0xa1 => {
                self.and_logical(self.c);
                self.pc += 1;
            } // ANA C
            0xa2 => {
                self.and_logical(self.d);
                self.pc += 1;
            } // ANA D
            0xa3 => {
                self.and_logical(self.e);
                self.pc += 1;
            } // ANA E
            0xa4 => {
                self.and_logical(self.h);
                self.pc += 1;
            } // ANA H
            0xa5 => {
                self.and_logical(self.l);
                self.pc += 1;
            } // ANA L
            0xa6 => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.and_logical(self.read_memory(mem_addr));
                self.pc += 1;
            } // ANA M
            0xa7 => {
                self.and_logical(self.a);
                self.pc += 1;
            } // ANA A
            0xa8 => {
                self.xor_logical(self.b);
                self.pc += 1;
            } // XRA B
            0xa9 => {
                self.xor_logical(self.c);
                self.pc += 1;
            } // XRA C
            0xaa => {
                self.xor_logical(self.d);
                self.pc += 1;
            } // XRA D
            0xab => {
                self.xor_logical(self.e);
                self.pc += 1;
            } // XRA E
            0xac => {
                self.xor_logical(self.h);
                self.pc += 1;
            } // XRA H
            0xad => {
                self.xor_logical(self.l);
                self.pc += 1;
            } // XRA L
            0xae => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.xor_logical(self.read_memory(mem_addr));
                self.pc += 1;
            } // XRA M
            0xaf => {
                self.xor_logical(self.a);
                self.pc += 1;
            } // XRA A
            0xb0 => {
                self.or_logical(self.b);
                self.pc += 1;
            } // ORA B
            0xb1 => {
                self.or_logical(self.c);
                self.pc += 1;
            } // ORA C
            0xb2 => {
                self.or_logical(self.d);
                self.pc += 1;
            } // ORA D
            0xb3 => {
                self.or_logical(self.e);
                self.pc += 1;
            } // ORA E
            0xb4 => {
                self.or_logical(self.h);
                self.pc += 1;
            } // ORA H
            0xb5 => {
                self.or_logical(self.l);
                self.pc += 1;
            } // ORA L
            0xb6 => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.or_logical(self.read_memory(mem_addr));
                self.pc += 1;
            } // ORA M
            0xb7 => {
                self.or_logical(self.a);
                self.pc += 1;
            } // ORA A
            0xb8 => {
                self.compare(self.b);
                self.pc += 1;
            } // CMP B
            0xb9 => {
                self.compare(self.c);
                self.pc += 1;
            } // CMP C
            0xba => {
                self.compare(self.d);
                self.pc += 1;
            } // CMP D
            0xbb => {
                self.compare(self.e);
                self.pc += 1;
            } // CMP E
            0xbc => {
                self.compare(self.h);
                self.pc += 1;
            } // CMP H
            0xbd => {
                self.compare(self.l);
                self.pc += 1;
            } // CMP L
            0xbe => {
                let mem_addr = Self::join_bytes(self.h, self.l);
                self.compare(self.read_memory(mem_addr));
                self.pc += 1;
            } // CMP M
            0xbf => {
                self.compare(self.a);
                self.pc += 1;
            } // CMP A
            0xc0 => {
                if !self.get_flag_bit(FlagBit::Zero) {
                    self.pc = self.pop();
                } else {
                    self.pc += 1;
                }
            } // RNZ
            0xc1 => {
                let bc = self.pop();
                self.b = ((bc >> 8) & 0xFF) as u8;
                self.c = (bc & 0xFF) as u8;
                self.pc += 1;
            } // POP B
            0xc2 => {
                if !self.get_flag_bit(FlagBit::Zero) {
                    self.pc = Self::join_bytes(self.read_memory(self.pc + 2), self.read_memory(self.pc + 1));
                } else {
                    self.pc += 3;
                }
            } // JNZ adr
            0xc3 => {
                self.pc = Self::join_bytes(self.read_memory(self.pc + 2), self.read_memory(self.pc + 1));
            }, // JMP adr
            0xc4 => {
                if !self.get_flag_bit(FlagBit::Zero) {
                    self.push(self.pc+3);
                    self.pc = Self::join_bytes(self.read_memory(self.pc+2), self.read_memory(self.pc+1));
                } else {
                    self.pc += 3;
                }
            } // CNZ adr
            0xc5 => {
                self.push(Self::join_bytes(self.b, self.c));
                self.pc += 1;
            } // PUSH B
            0xc6 => {
                self.add(self.read_memory(self.pc+1));
                self.pc += 2;
            } // ADI D8
            0xc7 => {
                self.push(self.pc + 1);
                self.pc = 0;
            }   // RST 0
            0xc8 => {
                if self.get_flag_bit(FlagBit::Zero) {
                    self.pc = self.pop();
                } else {
                    self.pc += 1;
                }
            } // RZ
            0xc9 => {
                self.pc = self.pop();
            } // RET
            0xca => {
                if self.get_flag_bit(FlagBit::Zero) {
                    self.pc = Self::join_bytes(self.read_memory(self.pc + 2), self.read_memory(self.pc + 1));
                } else {
                    self.pc += 3;
                }
            } // JZ adr
            0xcc => {
                if self.get_flag_bit(FlagBit::Zero) {
                    self.push(self.pc+3);
                    self.pc = Self::join_bytes(self.read_memory(self.pc+2), self.read_memory(self.pc+1));
                } else {
                    self.pc += 3;
                }
            } // CZ adr
            0xcd => {
                self.push(self.pc+3);
                self.pc = Self::join_bytes(self.read_memory(self.pc+2), self.read_memory(self.pc+1));
            } // CALL adr
            0xce => {
                self.add_with_carry(self.read_memory(self.pc+1));
                self.pc += 2;
            } // ACI D8 // A <- A + data + CY
            0xcf => {
                self.push(self.pc + 1);
                self.pc = 1 * 0x8;
            }   // RST 1
            0xd0 => {
                if !self.get_flag_bit(FlagBit::Carry) {
                    self.pc = self.pop();
                } else {
                    self.pc += 1;
                }
            } // RNC
            0xd1 => {
                let de = self.pop();
                self.d = ((de >> 8) & 0xFF) as u8;
                self.e = (de & 0xFF) as u8;
                self.pc += 1;
            } // POP D
            0xd2 => {
                if !self.get_flag_bit(FlagBit::Carry) {
                    self.pc = Self::join_bytes(self.read_memory(self.pc + 2), self.read_memory(self.pc + 1));
                } else {
                    self.pc += 3;
                }
            } // JNC adr
            0xd3 => {
                // self.write_port(self.read_memory(self.pc + 1), self.a);
                self.pc += 2;
            } // OUT D8
            0xd4 => {
                if !self.get_flag_bit(FlagBit::Carry) {
                    self.push(self.pc+3);
                    self.pc = Self::join_bytes(self.read_memory(self.pc+2), self.read_memory(self.pc+1));
                } else {
                    self.pc += 3;
                }
            } // CNC adr
            0xd5 => {
                self.push(Self::join_bytes(self.d, self.e));
                self.pc += 1;
            } // PUSH D
            0xd6 => {
                self.sub(self.read_memory(self.pc+1));
                self.pc += 2;
            } // SUI D8
            0xd7 => {
                self.push(self.pc + 1);
                self.pc = 2 * 0x8;
            } // RST 2
            0xd8 => {
                if self.get_flag_bit(FlagBit::Carry) {
                    self.pc = self.pop();
                } else {
                    self.pc += 1;
                }
            }   // RC
            0xda => {
                if self.get_flag_bit(FlagBit::Carry) {
                    self.pc = Self::join_bytes(self.read_memory(self.pc + 2), self.read_memory(self.pc + 1));
                } else {
                    self.pc += 3;
                }
            } // JC adr
            0xdb => {
                // self.a = self.read_port(self.read_memory(self.pc + 1));
                self.pc += 2;
            } // IN D8
            0xdc => {
                if self.get_flag_bit(FlagBit::Carry) {
                    self.push(self.pc+3);
                    self.pc = Self::join_bytes(self.read_memory(self.pc+2), self.read_memory(self.pc+1));
                } else {
                    self.pc += 3;
                }
            } // CC adr
            0xde => {
                self.sub_with_borrow(self.read_memory(self.pc+1));
                self.pc += 2;
            } // SBI D8
            0xdf => {
                self.push(self.pc + 1);
                self.pc = 3 * 0x8;
            }   // RST 3
            0xe0 => {
                if !self.get_flag_bit(FlagBit::Parity) {
                    self.pc = self.pop();
                } else {
                    self.pc += 1;
                }
            }   // RPO
            0xe1 => {
                let hl = self.pop();
                self.h = ((hl >> 8) & 0xFF) as u8;
                self.l = (hl & 0xFF) as u8;
                self.pc += 1;
            } // POP H
            0xe2 => {
                if !self.get_flag_bit(FlagBit::Parity) {
                    self.pc = Self::join_bytes(self.read_memory(self.pc + 2), self.read_memory(self.pc + 1));
                } else {
                    self.pc += 3;
                }
            } // JPO adr
            0xe3 => {
                std::mem::swap(&mut self.l, &mut self.memory[self.sp]);
                std::mem::swap(&mut self.h, &mut self.memory[self.sp + 1]);
                self.pc += 1;
            } // XTHL
            0xe4 => {
                if !self.get_flag_bit(FlagBit::Parity) {
                    self.push(self.pc+3);
                    self.pc = Self::join_bytes(self.read_memory(self.pc+2), self.read_memory(self.pc+1));
                } else {
                    self.pc += 3;
                }
            } // CPO adr
            0xe5 => {
                self.push(Self::join_bytes(self.h, self.l));
                self.pc += 1;
            } // PUSH H
            0xe6 => {
                self.and_logical(self.read_memory(self.pc+1));
                self.pc += 2;
            } // ANI D8
            0xe7 => {
                self.push(self.pc + 1);
                self.pc = 4 * 0x8;
            }   // RST 4
            0xe8 => {
                if self.get_flag_bit(FlagBit::Parity) {
                    self.pc = self.pop();
                } else {
                    self.pc += 1;
                }
            }   // RPE
            0xe9 => {
                self.pc = Self::join_bytes(self.h, self.l);
                // self.pc += 1; // ??????????
            }   // PCHL
            0xea => {
                if self.get_flag_bit(FlagBit::Parity) {
                    self.pc = Self::join_bytes(self.read_memory(self.pc + 2), self.read_memory(self.pc + 1));
                } else {
                    self.pc += 3;
                }
            } // JPE adr
            0xeb => {
                std::mem::swap(&mut self.h, &mut self.d);
                std::mem::swap(&mut self.l, &mut self.e);
                self.pc += 1;
            } // XCHG
            0xec => {
                if self.get_flag_bit(FlagBit::Parity) {
                    self.push(self.pc+3);
                    self.pc = Self::join_bytes(self.read_memory(self.pc+2), self.read_memory(self.pc+1));
                } else {
                    self.pc += 3;
                }
            } // CPE adr
            0xee => {
                self.xor_logical(self.read_memory(self.pc+1));
                self.pc += 2;
            } // XRI D8
            0xef => {
                self.push(self.pc+1);
                self.pc = 5 * 0x8;
            }   // RST 5
            0xf0 => {
                if !self.get_flag_bit(FlagBit::Sign) {
                    self.pc = self.pop();
                } else {
                    self.pc += 1;
                }
            }   // RP
            0xf1 => {
                let a_flags = self.pop();
                self.a = ((a_flags >> 8) & 0xFF) as u8;
                self.flags = (a_flags & 0xFF) as u8;
                self.pc += 1;
            } // POP PSW
            0xf2 => {
                if !self.get_flag_bit(FlagBit::Sign) {
                    self.pc = Self::join_bytes(self.read_memory(self.pc + 2), self.read_memory(self.pc + 1));
                } else {
                    self.pc += 3;
                }
            }   // JP adr
            0xf3 => {
                self.enable_interrupts = false;
                self.pc += 1;
            }   // DI
            0xf4 => {
                if !self.get_flag_bit(FlagBit::Sign) {
                    self.push(self.pc+3);
                    self.pc = Self::join_bytes(self.read_memory(self.pc+2), self.read_memory(self.pc+1));
                } else {
                    self.pc += 3;
                }
            } // CP adr
            0xf5 => {
                self.push(Self::join_bytes(self.a, self.flags));
                self.pc += 1;
            } // PUSH PSW
            0xf6 => {
                self.or_logical(self.read_memory(self.pc+1));
                self.pc += 2;
            } // ORI D8
            0xf7 => {
                self.push(self.pc+1);
                self.pc = 6 * 0x8;
            }   // RST 6
            0xf8 => {
                if self.get_flag_bit(FlagBit::Sign) {
                    self.pc = self.pop();
                } else {
                    self.pc += 1;
                }
            } // RM
            0xf9 => {
                self.sp = Self::join_bytes(self.h, self.l);
                self.pc += 1;
            }   // SPHL
            0xfa => {
                if self.get_flag_bit(FlagBit::Sign) {
                    self.pc = Self::join_bytes(self.read_memory(self.pc + 2), self.read_memory(self.pc + 1));
                } else {
                    self.pc += 3;
                }
            } // JM adr
            0xfb => {
                self.enable_interrupts = true;
                self.pc += 1;
            } // EI
            0xfc => {
                if self.get_flag_bit(FlagBit::Sign) {
                    self.push(self.pc+3);
                    self.pc = Self::join_bytes(self.read_memory(self.pc+2), self.read_memory(self.pc+1));
                } else {
                    self.pc += 3;
                }
            } // CM adr
            0xfe => {
                self.compare(self.read_memory(self.pc+1));
                self.pc += 2;
            } // CPI D8
            0xff => {
                self.push(self.pc+1);
                self.pc = 7 * 0x8;
            } // RST 7 // CALL $38
            _ => unreachable!(),
        }

        return INSTR_CYCLES[instr as usize] as usize;
    }
}
