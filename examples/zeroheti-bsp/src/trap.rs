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
        .word _start_DefaultHandler_trap // NC (1): _start_SupervisorSoft_trap
        .word _start_DefaultHandler_trap
        .word _start_MachineSoft_trap       // 3
        .word _start_DefaultHandler_trap
        .word _start_DefaultHandler_trap // NC (5): _start_SupervisorTimer_trap
        .word _start_DefaultHandler_trap
        .word _start_MachineTimer_trap      // 7
        .word _start_DefaultHandler_trap
        .word _start_DefaultHandler_trap // NC (9): _start_SupervisorExternal_trap
        .word _start_DefaultHandler_trap
        .word _start_MachineExternal_trap // 11

        // Fill up to 16 with `DefaultHandler`
        .rept 4
        .word _start_DefaultHandler_trap // 12..16
        .endr

        .word _start_Timer0Ovf_trap // 16
        .word _start_Timer0Cmp_trap // 17
        .word _start_Timer1Ovf_trap // 18
        .word _start_Timer1Cmp_trap // 19
        .word _start_Timer2Ovf_trap // 20
        .word _start_Timer2Cmp_trap // 21
        .word _start_Timer3Ovf_trap // 22
        .word _start_Timer3Cmp_trap // 23
        .word _start_Uart_trap      // 24
        .word _start_I2c_trap       // 25
        .word _start_Mbx_trap       // 26
        .word _start_Ext0_trap      // 27
        .word _start_Ext1_trap      // 28
        .word _start_Ext2_trap      // 29
        .word _start_Ext3_trap      // 30
        .word _start_Nmi_trap       // 31

    .option pop",
);
