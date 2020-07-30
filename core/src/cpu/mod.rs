mod alu;
mod decoder;
mod opcodes;
mod registers;

#[cfg(test)]
mod tests;

use self::opcodes::*;
use super::memory::Memory;
use registers::Registers;
use registers::R16;

pub struct CPU {
    reg: Registers,
    halted: bool,
    ime: bool,
    ime_next: bool,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            reg: Registers:: new(),
            halted: false,
            ime: false,
            ime_next: false,
        }
    }

    pub fn armed() -> Self {
        let mut cpu = CPU::new();
        cpu.reg.set_r16(R16::AF, 0x01B0);
        cpu.reg.set_r16(R16::BC, 0x0013);
        cpu.reg.set_r16(R16::DE, 0x00D8);
        cpu.reg.set_r16(R16::HL, 0x014D);
        cpu.reg.pc = 0x100;
        cpu.reg.sp = 0xFFFE;

        return cpu;
    }

    pub fn step(&mut self, mem: &mut dyn Memory) -> u32 {
        self.handle_ime();

        let mc = self.handle_interrupts(mem);

        if mc != 0 {
            mc * 4
        } else if self.halted {
            4
        } else {
            self.execute_next(mem) * 4
        }
    }

    fn handle_interrupts(&mut self, mem: &mut dyn Memory) -> u32 {
        if !self.ime && !self.halted {
            return 0;
        }

        let intfs = mem.read(0xFF0F) & 0x1F;
        let inte = mem.read(0xFFFF) & 0x1F;

        let filtered_intfs = inte & intfs;

        if filtered_intfs == 0 {
            return 0;
        }

        self.halted = false;

        if !self.ime {
            return 0;
        }

        let int_index = filtered_intfs.trailing_zeros() as u8;

        // Ack interrupt
        let new_intfs = intfs & !(1 << int_index);
        mem.write(0xFF0F, new_intfs);

        // Disable interrupts
        self.ime = false;

        self.stack_push(mem, self.reg.pc);

        self.reg.pc = self.get_int_routine_addr(int_index);

        // if int_index == 0 {
            // println!("INTERRUPT {:04x} EXECUTED!", self.reg.pc);
        // }

        4
    }

    fn get_int_routine_addr(&self, n: u8) -> u16 {
        0x0040 | ((n as u16) << 3)
    }

    fn handle_ime(&mut self) {
        if self.ime_next {
            self.ime = true;
            self.ime_next = false;
        }
    }

    fn dump(&mut self, mem: &mut dyn Memory) {
        use Opcode::*;

        self.reg.dump();
        let op1 = mem.read(self.reg.pc);
        let op2 = mem.read(self.reg.pc + 1);
        let op3 = mem.read(self.reg.pc + 2);

        let opcode = match decoder::decode(op1) {
            Some(PREFIX) => decoder::decode_prefix(op2),
            Some(opcode) => opcode,
            None => panic!("Unknown opcode 0x{:02X}!", op1),
        };

        println!("TIMA: {}  TAC: {:02X}  DIV: {}", mem.read(0xFF05), mem.read(0xFF07), mem.read(0xFF04));
        println!("IE: {:02X}  IF: {:02X}  IME: {}", mem.read(0xFFFF), mem.read(0xFF0F), self.ime);
        println!("LCDC: {:02X} STAT: {:02X} LY: {:02X}", mem.read(0xFF40), mem.read(0xFF41), mem.read(0xFF44));
        println!("00:{:04X}:  {:02X}{:02X}{:02X}  {:?}\n", self.reg.pc, op1,op2,op3, opcode);
    }

    fn execute_next(&mut self, mem: &mut dyn Memory) -> u32 {
        use registers::R16::*;
        use registers::R8::*;
        use Opcode::*;
        use Oper::*;

        // let instr_addr = self.reg.pc;

        // if self.reg.pc >= 0xC317 && self.reg.pc <= 0xC355 {
        // if self.reg.pc == 0x50 {
        //     self.dump(mem);
        // }

        let imm = self.imm_u8(mem);

        // if imm == 0xE6 { println!("{:04X} - {:02X}", instr_addr, imm); }
        let opcode = match decoder::decode(imm) {
            Some(PREFIX) => decoder::decode_prefix(self.imm_u8(mem)),
            Some(opcode) => opcode,
            None => panic!("Unknown opcode 0x{:02X}!", imm),
        };

        let cycles = match opcode {
            /*==============================*\
             * Aritmetic and Logic (8 bits) *
            \*==============================*/
            // ADC A, r8
            ADC(Reg8(A), Reg8(r)) => {
                let v = self.reg.get_r8(r);
                alu::add(self, v, true);
                1
            }
            // ADC A, (HL)
            ADC(Reg8(A), Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                let v = mem.read(a);
                alu::add(self, v, true);
                2
            }
            // ADC A, u8
            ADC(Reg8(A), ImmU8) => {
                let v = self.imm_u8(mem);
                alu::add(self, v, true);
                2
            }
            // ADD A, r8
            ADD(Reg8(A), Reg8(r)) => {
                let v = self.reg.get_r8(r);
                alu::add(self, v, false);
                1
            }
            // ADD A, (HL)
            ADD(Reg8(A), Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                let v = mem.read(a);
                alu::add(self, v, false);
                2
            }
            // ADD A, u8
            ADD(Reg8(A), ImmU8) => {
                let v = self.imm_u8(mem);
                alu::add(self, v, false);
                2
            }
            // AND A, r8
            AND(Reg8(A), Reg8(r)) => {
                let v = self.reg.get_r8(r);
                alu::and(self, v);
                1
            }
            // ADC A, (HL)
            AND(Reg8(A), Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                let v = mem.read(a);
                alu::and(self, v);
                2
            }
            // AND A, u8
            AND(Reg8(A), ImmU8) => {
                let v = self.imm_u8(mem);
                alu::and(self, v);
                2
            }
            // CP A, r8
            CP(Reg8(A), Reg8(r)) => {
                let v = self.reg.get_r8(r);
                alu::cp(self, v);
                1
            }
            // CP A, (HL)
            CP(Reg8(A), Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                let v = mem.read(a);
                alu::cp(self, v);
                2
            }
            // CP A, u8
            CP(Reg8(A), ImmU8) => {
                let v = self.imm_u8(mem);
                alu::cp(self, v);
                2
            }
            // DEC r8
            DEC(Reg8(r)) => {
                let v = self.reg.get_r8(r);
                let v = alu::dec(self, v);
                self.reg.set_r8(r, v);
                1
            }
            // DEC r8
            DEC(Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                let v = alu::dec(self, mem.read(a));
                mem.write(a, v);
                3
            }
            // INC r8
            INC(Reg8(r)) => {
                let v = self.reg.get_r8(r);
                let v = alu::inc(self, v);
                self.reg.set_r8(r, v);
                1
            }
            // INC r8
            INC(Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                let v = alu::inc(self, mem.read(a));
                mem.write(a, v);
                3
            }
            // OR A, r8
            OR(Reg8(A), Reg8(r)) => {
                let v = self.reg.get_r8(r);
                alu::or(self, v);
                1
            }
            // OR A, u8
            OR(Reg8(A), ImmU8) => {
                let v = self.imm_u8(mem);
                alu::or(self, v);
                2
            }
            // OR A, (HL)
            OR(Reg8(A), Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                let v = mem.read(a);
                alu::or(self, v);
                2
            }
            // SBC A, r8
            SBC(Reg8(A), Reg8(r)) => {
                let v = self.reg.get_r8(r);
                alu::sub(self, v, true);
                1
            }
            // SBC A, u8
            SBC(Reg8(A), ImmU8) => {
                let v = self.imm_u8(mem);
                alu::sub(self, v, true);
                2
            }
            // SBC A, (HL)
            SBC(Reg8(A), Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                let v = mem.read(a);
                alu::sub(self, v, true);
                2
            }
            // SUB A, r8
            SUB(Reg8(A), Reg8(r)) => {
                let v = self.reg.get_r8(r);
                alu::sub(self, v, false);
                1
            }
            // SUB A, u8
            SUB(Reg8(A), ImmU8) => {
                let v = self.imm_u8(mem);
                alu::sub(self, v, false);
                2
            }
            // SUB A, (HL)
            SUB(Reg8(A), Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                let v = mem.read(a);
                alu::sub(self, v, false);
                2
            }
            // XOR A, r8
            XOR(Reg8(A), Reg8(r)) => {
                let v = self.reg.get_r8(r);
                alu::xor(self, v);
                1
            }
            // XOR A, u8
            XOR(Reg8(A), ImmU8) => {
                let v = self.imm_u8(mem);
                alu::xor(self, v);
                2
            }
            // OR A, (HL)
            XOR(Reg8(A), Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                let v = mem.read(a);
                alu::xor(self, v);
                2
            }

            /*===============================*\
             * Aritmetic and Logic (16 bits) *
            \*===============================*/
            // DEC r16
            DEC(Reg16(r)) => {
                let v = self.reg.get_r16(r).wrapping_sub(1);
                self.reg.set_r16(r, v);
                1
            }
            // INC r16
            INC(Reg16(r)) => {
                let v = self.reg.get_r16(r).wrapping_add(1);
                self.reg.set_r16(r, v);
                1
            }
            // ADD HL, r16
            ADD(Reg16(HL), Reg16(r)) => {
                let v = self.reg.get_r16(r);
                alu::add_hl(self, v);
                2
            }

            /*=======================*\
            * Bit operations        *
            \*=======================*/
            // BIT u3, r8
            BIT(bit, Reg8(r)) => {
                let v = self.reg.get_r8(r);
                alu::bit(self, v, bit);
                2
            }
            // BIT u3, (HL)
            BIT(bit, Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                alu::bit(self, mem.read(a), bit);
                3
            }
            // RES u3, r8
            RES(bit, Reg8(r)) => {
                let v = self.reg.get_r8(r);
                let v = alu::res(self, v, bit);
                self.reg.set_r8(r, v);
                2
            }
            // RES u3, (HL)
            RES(bit, Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                let v = alu::res(self, mem.read(a), bit);
                mem.write(a, v);
                3
            }
            // SET u3, r8
            SET(bit, Reg8(r)) => {
                let v = self.reg.get_r8(r);
                let v = alu::set(self, v, bit);
                self.reg.set_r8(r, v);
                2
            }
            // SET u3, (HL)
            SET(bit, Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                let v = alu::set(self, mem.read(a), bit);
                mem.write(a, v);
                3
            }
            // SWAP r8
            SWAP(Reg8(r)) => {
                let v = self.reg.get_r8(r);
                let v = alu::swap(self, v);
                self.reg.set_r8(r, v);
                2
            }
            // SWAP u3, (HL)
            SWAP(Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                let v = alu::swap(self, mem.read(a));
                mem.write(a, v);
                4
            }
            /*==========================*\
             * Bit shift instructions   *
            \*==========================*/
            // RL r8
            RL(Reg8(r)) => {
                let v = self.reg.get_r8(r);
                let v = alu::rl(self, v);
                self.reg.set_r8(r, v);
                2
            }
            // RL u3, (HL)
            RL(Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                let v = alu::rl(self, mem.read(a));
                mem.write(a, v);
                4
            }
            // RLA
            RLA => {
                let v = self.reg.a;
                self.reg.a = alu::rl(self, v);
                self.reg.flags.z = false;
                1
            }
            // RLC r8
            RLC(Reg8(r)) => {
                let v = self.reg.get_r8(r);
                let v = alu::rlc(self, v);
                self.reg.set_r8(r, v);
                2
            }
            // RLC u3, (HL)
            RLC(Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                let v = alu::rlc(self, mem.read(a));
                mem.write(a, v);
                4
            }
            // RLCA
            RLCA => {
                let v = self.reg.a;
                self.reg.a = alu::rlc(self, v);
                self.reg.flags.z = false;
                1
            }
            // RR r8
            RR(Reg8(r)) => {
                let v = self.reg.get_r8(r);
                let v = alu::rr(self, v);
                self.reg.set_r8(r, v);
                2
            }
            // RR u3, (HL)
            RR(Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                let v = alu::rr(self, mem.read(a));
                mem.write(a, v);
                4
            }
            // RRA
            RRA => {
                let v = self.reg.a;
                self.reg.a = alu::rr(self, v);
                self.reg.flags.z = false;
                1
            }
            // RRC r8
            RRC(Reg8(r)) => {
                let v = self.reg.get_r8(r);
                let v = alu::rrc(self, v);
                self.reg.set_r8(r, v);
                2
            }
            // RRC u3, (HL)
            RRC(Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                let v = alu::rrc(self, mem.read(a));
                mem.write(a, v);
                4
            }
            // RRCA
            RRCA => {
                let v = self.reg.a;
                self.reg.a = alu::rrc(self, v);
                self.reg.flags.z = false;
                1
            }
            // SLA r8
            SLA(Reg8(r)) => {
                let v = self.reg.get_r8(r);
                let v = alu::sla(self, v);
                self.reg.set_r8(r, v);
                2
            }
            // SLA (HL)
            SLA(Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                let v = alu::sla(self, mem.read(a));
                mem.write(a, v);
                4
            }
            // SRA r8
            SRA(Reg8(r)) => {
                let v = self.reg.get_r8(r);
                let v = alu::sra(self, v);
                self.reg.set_r8(r, v);
                2
            }
            // SRA (HL)
            SRA(Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                let v = alu::sra(self, mem.read(a));
                mem.write(a, v);
                4
            }
            // SRL r8
            SRL(Reg8(r)) => {
                let v = self.reg.get_r8(r);
                let v = alu::srl(self, v);
                self.reg.set_r8(r, v);
                2
            }
            // SR: (HL)
            SRL(Mem(HL)) => {
                let a = self.reg.get_r16(HL);
                let v = alu::srl(self, mem.read(a));
                mem.write(a, v);
                4
            }
            /*==========================*\
             * Load instructions        *
            \*==========================*/
            // LD r8, r8
            LD(Reg8(dr), Reg8(sr)) => {
                let v = self.reg.get_r8(sr);
                self.reg.set_r8(dr, v);
                1
            }
            // LD r8, u8
            LD(Reg8(dr), ImmU8) => {
                let v = self.imm_u8(mem);
                self.reg.set_r8(dr, v);
                2
            }
            // LD (r16), u8
            LD(Mem(ar), ImmU8) => {
                let a = self.reg.get_r16(ar);
                let v = self.imm_u8(mem);
                mem.write(a, v);
                3
            }
            // LD (r16), r8
            LD(Mem(ar), Reg8(sr)) => {
                let a = self.reg.get_r16(ar);
                let v = self.reg.get_r8(sr);
                mem.write(a, v);
                2
            }
            // LD r8, (r16)
            LD(Reg8(dr), Mem(ar)) => {
                let a = self.reg.get_r16(ar);
                let v = mem.read(a);
                self.reg.set_r8(dr, v);
                2
            }
            // LD r16, u16
            LD(Reg16(dr), ImmU16) => {
                let v = self.imm_u16(mem);
                self.reg.set_r16(dr, v);
                3
            }
            // LD A, (u16)
            LD(Reg8(A), MemImmU16) => {
                let a = self.imm_u16(mem);
                let v = mem.read(a);
                self.reg.a = v;
                4
            }
            // LD (u16), A
            LD(MemImmU16, Reg8(A)) => {
                let a = self.imm_u16(mem);
                mem.write(a, self.reg.a);
                4
            }
            // LD A, (0xFF00+u8)
            LD(Reg8(A), ZMemImmU8) => {
                let a = self.imm_u8(mem) as u16 | 0xFF00;
                self.reg.a = mem.read(a);
                3
            }
            // LD (0xFF00+u8), A
            LD(ZMemImmU8, Reg8(A)) => {
                let a = self.imm_u8(mem) as u16 | 0xFF00;
                mem.write(a, self.reg.a);
                3
            }
            // LD A, (0xFF00+C)
            LD(Reg8(A), ZMem(C)) => {
                let a = self.reg.c as u16 | 0xFF00;
                self.reg.a = mem.read(a);
                3
            }
            // LD (0xFF00+C), A
            LD(ZMem(C), Reg8(A)) => {
                let a = self.reg.c as u16 | 0xFF00;
                mem.write(a, self.reg.a);
                3
            }
            // LDD A, (HL)
            LDD(Reg8(A), Mem(HL)) => {
                let addr = self.reg.get_r16(HL);
                self.reg.a = mem.read(addr);
                self.reg.set_r16(HL, addr.wrapping_sub(1));
                2
            }
            // LDD A, (HL)
            LDD(Mem(HL), Reg8(A)) => {
                let addr = self.reg.get_r16(HL);
                mem.write(addr, self.reg.a);
                self.reg.set_r16(HL, addr.wrapping_sub(1));
                2
            }
            // LDI A, (HL)
            LDI(Reg8(A), Mem(HL)) => {
                let addr = self.reg.get_r16(HL);
                self.reg.a = mem.read(addr);
                self.reg.set_r16(HL, addr.wrapping_add(1));
                2
            }
            // LDI A, (HL)
            LDI(Mem(HL), Reg8(A)) => {
                let addr = self.reg.get_r16(HL);
                mem.write(addr, self.reg.a);
                self.reg.set_r16(HL, addr.wrapping_add(1));
                2
            }

            /*==========================*\
             * Jumps and Subroutines    *
            \*==========================*/
            // JR (Cond) i8
            JR(cond, ImmI8) => {
                let o = self.imm_i8(mem);
                let a = self.add_u16_i8(self.reg.pc, o);
                if self.check_cond(cond) {
                    self.reg.pc = a;
                    3
                } else {
                    2
                }
            }
            // JP (Cond) u16
            JP(cond, ImmU16) => {
                let a = self.imm_u16(mem);
                if self.check_cond(cond.clone()) {
                    self.reg.pc = a;
                    4
                } else {
                    3
                }
            }
            // JP (Cond) u16
            JP(Cond::Always, Reg16(HL)) => {
                self.reg.pc = self.reg.get_r16(HL);
                4
            }
            // CALL (Cond) u16
            CALL(cond, ImmU16) => {
                let addr = self.imm_u16(mem);
                let ret_addr = self.reg.pc;

                if self.check_cond(cond) {
                    self.stack_push(mem, ret_addr);
                    self.reg.pc = addr;
                    4
                } else {
                    3
                }
            }
            // RET
            RET(Cond::Always) => {
                let addr = self.stack_pop(mem);
                self.reg.pc = addr;
                4
            }
            // RET Cond
            RET(cond) => {
                if self.check_cond(cond) {
                    let addr = self.stack_pop(mem);
                    self.reg.pc = addr;
                    5
                } else {
                    2
                }
            }
            // RETI
            RETI => {
                let addr = self.stack_pop(mem);
                self.reg.pc = addr;
                self.ime_next = true;
                4
            }
            // RST u8
            RST(vec) => {
                let ret_addr = self.reg.pc;
                self.stack_push(mem, ret_addr);
                self.reg.pc = vec as u16;
                4
            }

            /*==========================*\
             * Stack                     *
            \*==========================*/
            // LD SP, HL
            LD(Reg16(SP), Reg16(HL)) => {
                let v = self.reg.get_r16(HL);
                self.reg.set_r16(SP, v);
                2
            }
            // ADD SP, i8
            ADD(Reg16(SP), ImmI8) => {
                let v = self.imm_i8(mem);
                alu::add_sp(self, v);
                4
            }
            // LD (u16), SP
            LD(MemImmU16, Reg16(SP)) => {
                let a = self.imm_u16(mem);
                mem.write_word(a, self.reg.get_r16(SP));
                5
            }
            // LD HL, SP+i8
            LD(Reg16(HL), SPImmI8) => {
                let a = self.reg.sp;
                let b = self.imm_i8(mem) as i16 as u16;
                let r = a.wrapping_add(b);
                self.reg.flags.c = (a & 0x00ff) + (b & 0x00ff) > 0x00ff;
                self.reg.flags.h = (a & 0x000f) + (b & 0x000f) > 0x000f;
                self.reg.flags.n = false;
                self.reg.flags.z = false;
                self.reg.set_r16(HL, r);

                3
            }
            PUSH(Reg16(r)) => {
                let v = self.reg.get_r16(r);
                self.stack_push(mem, v);
                4
            }
            POP(Reg16(r)) => {
                let v = self.stack_pop(mem);
                self.reg.set_r16(r, v);
                4
            }

            /*==========================*\
             * Misc                     *
            \*==========================*/
            NOP => 1,
            HALT => {
                self.halted = true;
                1
            }
            DI => {
                self.ime_next = false;
                1
            }
            EI => {
                self.ime_next = true;
                1
            }
            STOP => 1,
            DAA => {
                alu::daa(self);
                1
            }
            CPL => {
                self.reg.a = !self.reg.a;
                self.reg.flags.h = true;
                self.reg.flags.n = true;
                1
            }
            CCF => {
                self.reg.flags.c = !self.reg.flags.c;
                self.reg.flags.n = false;
                self.reg.flags.h = false;
                1
            }
            SCF => {
                self.reg.flags.c = true;
                self.reg.flags.n = false;
                self.reg.flags.h = false;
                1
            }
            opcode => panic!("Unimplemented opcode: {:?}", opcode),
        };

        cycles * 4
    }

    fn check_cond(&self, cond: Cond) -> bool {
        match cond {
            Cond::Always => true,
            Cond::Z => self.reg.flags.z,
            Cond::NZ => !self.reg.flags.z,
            Cond::C => self.reg.flags.c,
            Cond::NC => !self.reg.flags.c,
        }
    }

    fn imm_u8(&mut self, mem: &dyn Memory) -> u8 {
        let v = mem.read(self.reg.pc);
        self.reg.pc = self.reg.pc.wrapping_add(1);

        v
    }

    fn imm_i8(&mut self, mem: &dyn Memory) -> i8 {
        let v = mem.read(self.reg.pc);
        self.reg.pc = self.reg.pc.wrapping_add(1);

        v as i8
    }

    fn imm_u16(&mut self, mem: &dyn Memory) -> u16 {
        let v = mem.read_word(self.reg.pc);
        self.reg.pc = self.reg.pc.wrapping_add(2);

        v
    }

    fn stack_push(&mut self, mem: &mut dyn Memory, v: u16) {
        self.reg.sp = self.reg.sp.wrapping_sub(2);
        mem.write_word(self.reg.sp, v);
    }

    fn stack_pop(&mut self, mem: &mut dyn Memory) -> u16 {
        let v = mem.read_word(self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(2);
        v
    }

    fn add_u16_i8(&self, a: u16, b: i8) -> u16 {
        ((a as u32 as i32) + b as i32) as u16
    }
}
