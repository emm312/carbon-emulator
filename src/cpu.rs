pub struct CPU {
    rom: Vec<u8>,
    pc: u8,
    page: u8,
    ram: Vec<u8>,
    regs: Regs,
    acc: u8,
    zero: bool,
    msb: bool,
    carry: bool,
    pub halted: bool,
    pub ports: Ports,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            rom: vec![0; 32*32],
            pc: 0,
            page: 0,
            ram: vec![0; 32],
            regs: Regs::new(),
            acc: 0,
            zero: false,
            msb: false,
            carry: false,
            halted: false,
            ports: Ports::new(),
        }
    }
    pub fn load_rom(&mut self, rom: Vec<u8>) {
        for (pos, elem) in rom.iter().enumerate() {
            self.rom[pos] = *elem;
        }
    }
    fn get_imm(&mut self) -> u8 {
        let ret = self.rom[(self.pc + (self.page * 32)) as usize];
        self.inc_pc();
        ret
    }

    #[inline]
    fn inc_pc(&mut self) {
        if self.pc == 32 {
            self.page += 1;
            self.pc = 0;
        } else {
            self.pc += 1;
        }
    }

    pub fn exec_opcode(&mut self) {
        let word = self.rom[(self.pc + (self.page * 32)) as usize];
        let opcode = word >> 3;
        let operand = (word & 0b00000111) as usize;
        self.inc_pc();
        match opcode {
            0 => (),
            0b11111 => self.halted = true,
            1 => {
                let res = self.regs.read(operand) + self.acc;
                self.set_flags(
                    is_ovf_add(self.regs.read(operand), self.acc, res),
                    res & 0b10000000 != 0,
                    res == 0,
                );
                self.acc = res;
            } // add
            2 => {
                let res = self.acc - self.regs.read(operand);
                self.set_flags(
                    is_ovf_sub(self.regs.read(operand), self.acc, res),
                    res & 0b10000000 != 0,
                    res == 0,
                );
                self.acc = res;
            } // sub
            3 => {
                let res = self.regs.read(operand) - self.acc;
                self.set_flags(
                    is_ovf_sub(self.regs.read(operand), self.acc, res),
                    res & 0b10000000 != 0,
                    res == 0,
                );
                self.acc = res;
            } // bsub
            4 => {
                self.acc |= self.regs.read(operand);
                self.set_flags(false, self.acc & 0b10000000 != 0, self.acc == 0);
            } // or
            5 => {
                self.acc = !(self.acc | self.regs.read(operand));
                self.set_flags(false, self.acc & 0b10000000 != 0, self.acc == 0);
            } // nor
            6 => {
                self.acc &= self.regs.read(operand);
                self.set_flags(false, self.acc & 0b10000000 != 0, self.acc == 0);
            } // and
            7 => {
                self.acc = !(self.acc & self.regs.read(operand));
                self.set_flags(false, self.acc & 0b10000000 != 0, self.acc == 0);
            } // nand
            8 => {
                self.acc ^= self.regs.read(operand);
                self.set_flags(false, self.acc & 0b10000000 != 0, self.acc == 0);
            } // xor
            9 => {
                self.acc = !(self.acc ^ self.regs.read(operand));
                self.set_flags(false, self.acc & 0b10000000 != 0, self.acc == 0);
            } // xnor
            10 => {
                let imm = self.get_imm();
                self.regs.write(operand, imm)
            } // ldi
            11 => {
                let res = self.regs.read(operand) + self.acc;
                self.set_flags(
                    is_ovf_add(self.regs.read(operand), self.acc, res),
                    res & 0b10000000 != 0,
                    res == 0,
                );
                self.regs.write(operand, res);
            } // adr
            12 => self.acc = self.regs.read(operand), // rld
            13 => self.regs.write(operand, self.acc), // rst
            14 => self.ram[operand] = self.acc,       // mst
            15 => self.acc = self.ram[operand],       // mld
            16 => {
                if self.eval_cond(operand as u8) {
                    self.page = (self.get_imm()) >> 3;
                } else {
                    self.inc_pc();
                }
            } // ics
            17 => {
                self.pc = self.regs.read(operand) & 0b11111;
            } // jid
            18 => {
                if self.eval_cond(operand as u8) {
                    self.pc = (self.get_imm() & 0b11111) >> 3;
                } else {
                    self.inc_pc();
                }
            } // brc
            19 => {
                let res = self.acc - 1;
                self.set_flags(
                    is_ovf_sub(self.acc, 1, res),
                    res & 0b10000000 != 0,
                    res == 0,
                );
                self.acc = res;
            } // dec
            20 => {
                let res = self.regs.read(operand) - self.acc;
                self.set_flags(
                    is_ovf_sub(self.regs.read(operand), self.acc, res),
                    res & 0b10000000 != 0,
                    res == 0,
                );
            } // cmp
            21 => {
                self.acc = self.acc >> operand;
                self.set_flags(false, self.acc & 0b10000000 != 0, self.acc == 0);
            } // bsr
            22 => {
                self.acc = self.acc << operand;
                self.set_flags(false, self.acc & 0b10000000 != 0, self.acc == 0);
            } // bsl
            23 => {
                self.ports.write(operand, self.acc);
            } // pst
            24 => {
                self.acc = self.ports.read(operand);
            } // pld
            25 => {
                let res = self.acc + 1;
                self.set_flags(
                    is_ovf_add(self.acc, 1, res),
                    res & 0b10000000 != 0,
                    res == 0,
                );
                self.acc = res;
            } // inc
            _ => todo!(
                "{:#010b}: {} {} {}",
                word,
                self.pc + self.page * 32,
                self.pc,
                self.page
            ),
        }
    }

    fn eval_cond(&self, operand: u8) -> bool {
        match operand {
            0b00000001 => self.zero,
            0b00000010 => !self.zero,
            0b00000011 => self.msb,
            0b00000100 => !self.msb,
            0b00000101 => self.carry,
            0b00000110 => !self.carry,
            0b00000111 => true,
            _ => unreachable!("{:#010b}: {}", operand, self.pc),
        }
    }

    fn set_flags(&mut self, ovf: bool, msb: bool, zr: bool) {
        self.carry = ovf;
        self.msb = msb;
        self.zero = zr;
    }
}

fn is_ovf_add(a: u8, b: u8, sum: u8) -> bool {
    if a as u32 + b as u32 != sum as u32 {
        true
    } else {
        false
    }
}

fn is_ovf_sub(a: u8, b: u8, sum: u8) -> bool {
    if a as u32 - b as u32 != sum as u32 {
        true
    } else {
        false
    }
}

pub struct Ports {
    console_buf: Vec<u8>
}

impl Ports {
    pub fn new() -> Ports {
        Ports { console_buf: Vec::new() }
    }
    #[inline]
    pub fn read(&mut self, _: usize) -> u8 {
        0
    }
    #[inline]
    pub fn write(&mut self, pos: usize, val: u8) {
        match pos {
            7 => self.console_buf.push(val),
            _ => (),
        }
    }
    pub fn flush(&mut self) {
        let mut str = String::new();
        for elem in &self.console_buf {
            str += &format!("{}\n", elem);
        }
        print!("{}", str);
        self.console_buf = vec![];
    }
}

struct Regs {
    regs: Vec<u8>,
}

impl Regs {
    pub fn new() -> Regs {
        Regs { regs: vec![0; 7] }
    }

    pub fn read(&self, reg: usize) -> u8 {
        if reg == 0 {
            0
        } else {
            self.regs[reg - 1]
        }
    }

    pub fn write(&mut self, reg: usize, val: u8) {
        if reg != 0 {
            self.regs[reg - 1] = val
        }
    }
}
