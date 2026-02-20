//! Basic MMIO operations
//!
//! # Safety
//!
//! Use of these methods is inherently unsafe, as hardware can do whatever. The
//! only way to assert safety is to not depend on this module.

// We trust users of MMIO to use these functions responsibly
#![allow(clippy::not_unsafe_ptr_arg_deref)]

#[inline(always)]
pub fn read_u8(addr: usize) -> u8 {
    // Safety: unaligned reads may fail to produce expected results.
    unsafe { core::ptr::read_volatile(addr as *const _) }
}

/// Reads the masked bits from the register
#[inline(always)]
pub fn read_u8_masked(addr: usize, mask: u8) -> u8 {
    // Safety: unaligned reads may fail to produce expected results.
    unsafe { core::ptr::read_volatile(addr as *const u8) & mask }
}

#[inline(always)]
pub fn write_u8(addr: usize, val: u8) {
    // Safety: unaligned reads may fail to produce expected results.
    unsafe { core::ptr::write_volatile(addr as *mut _, val) }
}

#[inline(always)]
pub fn write_u16(addr: usize, val: u16) {
    // Safety: unaligned reads may fail to produce expected results.
    unsafe { core::ptr::write_volatile(addr as *mut _, val) }
}

#[inline(always)]
pub fn read_u32(addr: usize) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const _) }
}

#[inline(always)]
pub fn read_u32p(ptr: *const u32) -> u32 {
    unsafe { core::ptr::read_volatile(ptr) }
}

#[inline(always)]
pub fn write_u32(addr: usize, val: u32) {
    write_u32p(addr as *mut _, val)
}

#[inline(always)]
pub fn write_u32p(ptr: *mut u32, val: u32) {
    unsafe { core::ptr::write_volatile(ptr, val) }
}

#[inline(always)]
pub fn modify_u32(addr: usize, val: u32, mask: u32, bit_pos: usize) {
    let mut tmp = read_u32(addr);
    tmp &= !(mask << bit_pos); // Clear bitfields
    write_u32(addr, tmp | (val << bit_pos));
}

#[inline(always)]
pub fn mask_u32(addr: usize, mask: u32) {
    mask_u32p(addr as *mut u32, mask)
}

#[inline(always)]
pub fn mask_u32p(ptr: *mut u32, mask: u32) {
    let r = unsafe { core::ptr::read_volatile(ptr) };
    unsafe { core::ptr::write_volatile(ptr, r | mask) }
}

/// Unmasks specified bits from given register
#[inline(always)]
pub fn unmask_u32(addr: usize, unmask: u32) {
    unmask_u32p(addr as *mut _, unmask);
}

/// Unmasks specified bits from given register
#[inline(always)]
pub fn unmask_u32p(ptr: *mut u32, unmask: u32) {
    let r = unsafe { core::ptr::read_volatile(ptr) };
    unsafe { core::ptr::write_volatile(ptr as *mut _, r & !unmask) }
}

#[inline(always)]
pub fn toggle_u32(addr: usize, toggle_bits: u32) {
    let mut r = read_u32(addr);
    r ^= toggle_bits;
    write_u32(addr, r);
}

#[inline(always)]
pub fn mask_u8(addr: usize, mask: u8) {
    // Safety: unaligned reads/writes may fail to produce expected results.
    let r = unsafe { core::ptr::read_volatile(addr as *const u8) };
    unsafe { core::ptr::write_volatile(addr as *mut u8, r | mask) }
}

/// Unmasks specified bits from given register
#[inline(always)]
pub fn unmask_u8(addr: usize, unmask: u8) {
    // Safety: unaligned reads/writes may fail to produce expected results.
    let r = unsafe { core::ptr::read_volatile(addr as *const u8) };
    unsafe { core::ptr::write_volatile(addr as *mut u8, r & !unmask) }
}
