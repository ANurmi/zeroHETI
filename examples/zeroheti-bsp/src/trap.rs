#[cfg(feature = "rt")]
use core::arch::global_asm;

#[cfg(feature = "rt")]
#[unsafe(export_name = "_setup_interrupts")]
fn setup_interrupt_vector() {
    use crate::register::{mintthresh, mtvec, mtvt};

    // Set the trap vector
    unsafe {
        unsafe extern "C" {
            fn _vector_table();
        }

        // Set all the trap vectors for good measure
        let bits = _vector_table as usize;
        mtvec::write(mtvec::Mtvec::from_bits(
            bits | mtvt::TrapMode::Clic as usize,
        ));
        mtvt::write(bits, mtvt::TrapMode::Clic);

        mintthresh::write(0x00.into());
    }
}

// The vector table
//
// N.b. vectors length must be exactly 0x80
#[cfg(feature = "rt")]
global_asm!(
    "
.section .vectors, \"ax\"
    .global _vector_table
    .type _vector_table, @function

    .option push
    // Antti told me that this device needs 2^8 alignment
    .p2align 8
    .option norelax
    .option norvc

    _vector_table:
        // Use [0] as exception entry point
        j _start_trap
        // [1..=16] are standard
        .word _start_SupervisorSoft_trap    // 1
        .word _start_DefaultHandler_trap
        .word _start_MachineSoft_trap       // 3
        .word _start_DefaultHandler_trap
        .word _start_SupervisorTimer_trap   // 5
        .word _start_DefaultHandler_trap
        .word _start_MachineTimer_trap      // 7
        .word _start_DefaultHandler_trap
        .word _start_SupervisorExternal_trap // 9
        .word _start_DefaultHandler_trap
        .word _start_MachineExternal_trap   // 11

        // Fill the rest with `DefaultHandler`
        .rept 53
        .word _start_DefaultHandler_trap // 12..64
        .endr

        // TODO: add remaining missing interrupts

    .option pop",
);
