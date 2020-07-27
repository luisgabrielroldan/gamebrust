use super::registers::R16;
use super::CPU;

pub fn daa(cpu: &mut CPU) {
    let mut a = cpu.reg.a;
    let mut adjust = if cpu.reg.flags.c { 0x60 } else { 0x00 };
    if cpu.reg.flags.h {
        adjust |= 0x06;
    };
    if !cpu.reg.flags.n {
        if a & 0x0F > 0x09 {
            adjust |= 0x06;
        };
        if a > 0x99 {
            adjust |= 0x60;
        };
        a = a.wrapping_add(adjust);
    } else {
        a = a.wrapping_sub(adjust);
    }

    cpu.reg.flags.c = adjust >= 0x60;
    cpu.reg.flags.h = false;
    cpu.reg.flags.z = a == 0;
    cpu.reg.a = a;
}

pub fn add(cpu: &mut CPU, b: u8, carry: bool) {
    let c = if carry && cpu.reg.flags.c { 1 } else { 0 };
    let a = cpu.reg.a;
    let r = a.wrapping_add(b).wrapping_add(c);
    cpu.reg.flags.z = r == 0;
    cpu.reg.flags.h = (a & 0xF) + (b & 0xF) + c > 0xF;
    cpu.reg.flags.n = false;
    cpu.reg.flags.c = (a as u16) + (b as u16) + (c as u16) > 0xFF;
    cpu.reg.a = r;
}

pub fn add_hl(cpu: &mut CPU, b: u16) {
    let a = cpu.reg.get_r16(R16::HL);
    let r = a.wrapping_add(b);
    cpu.reg.flags.h = (a & 0x07FF) + (b & 0x07FF) > 0x07FF;
    cpu.reg.flags.n = false;
    cpu.reg.flags.c = a > 0xFFFF - b;
    cpu.reg.set_r16(R16::HL, r);
}

pub fn add_sp(cpu: &mut CPU, b: i8) {
    let a = cpu.reg.get_r16(R16::SP) as u32 as i32;
    let b = b as i32;
    let r = (a + b) as u16;
    cpu.reg.flags.h = (a & 0x07FF) + (b & 0x07FF) > 0x07FF;
    cpu.reg.flags.c = a > 0xFFFF - b;
    cpu.reg.set_r16(R16::SP, r);
}

pub fn sub(cpu: &mut CPU, b: u8, carry: bool) {
    let c = if carry && cpu.reg.flags.c { 1 } else { 0 };
    let a = cpu.reg.a;
    let r = a.wrapping_sub(b).wrapping_sub(c);
    cpu.reg.flags.z = r == 0;
    cpu.reg.flags.h = (a & 0x0F) < (b & 0x0F) + c;
    cpu.reg.flags.n = true;
    cpu.reg.flags.c = (a as u16) < (b as u16) + (c as u16);
    cpu.reg.a = r;
}

pub fn rl(cpu: &mut CPU, a: u8) -> u8 {
    let c = a & 0x80 == 0x80;
    let r = (a << 1) | (if cpu.reg.flags.c { 1 } else { 0 });
    rot_flags_update(cpu, r, c);
    r
}

pub fn rr(cpu: &mut CPU, a: u8) -> u8 {
    let c = a & 0x01 == 0x01;
    let r = (a >> 1) | (if cpu.reg.flags.c { 0x80 } else { 0 });
    rot_flags_update(cpu, r, c);
    r
}

pub fn rlc(cpu: &mut CPU, a: u8) -> u8 {
    let c = a & 0x80 == 0x80;
    let r = (a << 1) | (if c { 1 } else { 0 });
    rot_flags_update(cpu, r, c);
    r
}

pub fn rrc(cpu: &mut CPU, a: u8) -> u8 {
    let c = a & 0x01 == 0x01;
    let r = (a >> 1) | (if c { 0x80 } else { 0 });
    rot_flags_update(cpu, r, c);
    r
}

pub fn sla(cpu: &mut CPU, a: u8) -> u8 {
    let c = a & 0x80 == 0x80;
    let r = a << 1;
    rot_flags_update(cpu, r, c);
    r
}

pub fn sra(cpu: &mut CPU, a: u8) -> u8 {
    let c = a & 0x01 == 0x01;
    let r = (a >> 1) | (a & 0x80);
    rot_flags_update(cpu, r, c);
    r
}

pub fn srl(cpu: &mut CPU, a: u8) -> u8 {
    let c = a & 0x01 == 0x01;
    let r = a >> 1;
    rot_flags_update(cpu, r, c);
    r
}

pub fn inc(cpu: &mut CPU, a: u8) -> u8 {
    let r = a.wrapping_add(1);
    cpu.reg.flags.z = r == 0;
    cpu.reg.flags.n = false;
    cpu.reg.flags.h = (a & 0x0F) + 1 > 0x0F;

    r
}

pub fn dec(cpu: &mut CPU, a: u8) -> u8 {
    let r = a.wrapping_sub(1);
    cpu.reg.flags.z = r == 0;
    cpu.reg.flags.h = (a & 0x0F) == 0;
    cpu.reg.flags.n = true;

    r
}

pub fn and(cpu: &mut CPU, b: u8) {
    let a = cpu.reg.a & b;
    cpu.reg.flags.z = a == 0;
    cpu.reg.flags.n = false;
    cpu.reg.flags.h = true;
    cpu.reg.flags.c = false;
    cpu.reg.a = a;
}

pub fn or(cpu: &mut CPU, b: u8) {
    let a = cpu.reg.a | b;
    cpu.reg.flags.z = a == 0;
    cpu.reg.flags.n = false;
    cpu.reg.flags.h = false;
    cpu.reg.flags.c = false;
    cpu.reg.a = a;
}

pub fn xor(cpu: &mut CPU, b: u8) {
    let a = cpu.reg.a ^ b;
    cpu.reg.flags.z = a == 0;
    cpu.reg.flags.n = false;
    cpu.reg.flags.h = false;
    cpu.reg.flags.c = false;
    cpu.reg.a = a;
}

pub fn swap(cpu: &mut CPU, a: u8) -> u8 {
    cpu.reg.flags.z = a == 0;
    cpu.reg.flags.n = false;
    cpu.reg.flags.h = false;
    cpu.reg.flags.c = false;
    (a >> 4) | (a << 4)
}

pub fn res(_cpu: &mut CPU, a: u8, b: u8) -> u8 {
    a & !(1 << (b as u32))
}

pub fn set(_cpu: &mut CPU, a: u8, b: u8) -> u8 {
    a | (1 << (b as u32))
}

pub fn bit(cpu: &mut CPU, a: u8, b: u8) {
    let r = a & (1 << (b as u32)) == 0;
    cpu.reg.flags.z = r;
    cpu.reg.flags.n = false;
    cpu.reg.flags.h = true;
}

pub fn cp(cpu: &mut CPU, v: u8) {
    let a = cpu.reg.a;
    sub(cpu, v, false);
    cpu.reg.a = a;
}

fn rot_flags_update(cpu: &mut CPU, r: u8, carry: bool) {
    cpu.reg.flags.h = false;
    cpu.reg.flags.n = false;
    cpu.reg.flags.z = r == 0;
    cpu.reg.flags.c = carry;
}
