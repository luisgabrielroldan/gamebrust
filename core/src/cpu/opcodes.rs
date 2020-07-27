use super::registers::{R16, R8};

#[derive(Debug)]
pub enum Oper {
    ImmI8,
    ImmU16,
    ImmU8,
    Mem(R16),
    MemImmU16,
    Reg16(R16),
    Reg8(R8),
    SPImmI8,
    ZMem(R8),
    ZMemImmU8,
}

#[derive(Debug)]
pub enum Cond {
    Always,
    Z,
    NZ,
    C,
    NC,
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Opcode {
    PREFIX,
    ADC(Oper, Oper),
    ADD(Oper, Oper),
    AND(Oper, Oper),
    BIT(u8, Oper),
    CALL(Cond, Oper),
    CCF,
    CP(Oper, Oper),
    CPL,
    DAA,
    DEC(Oper),
    DI,
    EI,
    HALT,
    INC(Oper),
    JP(Cond, Oper),
    JR(Cond, Oper),
    LD(Oper, Oper),
    LDD(Oper, Oper),
    LDI(Oper, Oper),
    NOP,
    OR(Oper, Oper),
    POP(Oper),
    PUSH(Oper),
    RES(u8, Oper),
    RET(Cond),
    RETI,
    RL(Oper),
    RLA,
    RLC(Oper),
    RLCA,
    RR(Oper),
    RRA,
    RRC(Oper),
    RRCA,
    RST(u8),
    SBC(Oper, Oper),
    SCF,
    SET(u8, Oper),
    SLA(Oper),
    SRA(Oper),
    SRL(Oper),
    STOP,
    SUB(Oper, Oper),
    SWAP(Oper),
    XOR(Oper, Oper),
}
