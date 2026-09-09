#[cfg(feature = "rt")]
use core::arch::global_asm;

#[cfg(feature = "rt")]
#[riscv_rt::setup_interrupts]
fn setup_interrupt_vector() {
    use crate::register::{mintthresh, mtvec, mtvt};

    // Set the trap vector
    unsafe {
        unsafe extern "C" {
            fn _vector_table();
        }

        // Set all the trap vectors for good measure
        let bits = _vector_table as *const () as usize;
        mtvec::write(mtvec::Mtvec::from_bits(
            bits | mtvt::TrapMode::Clic as usize,
        ));
        mtvt::write(bits, mtvt::TrapMode::Clic);

        mintthresh::write(0x00.into());
    }
}

// The vector table
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

        .word _start_Mbx_trap       // 16
        .word _start_SpiEvent0_trap // 17
        .word _start_SpiEvent1_trap // 18
        .word _start_I2c0_trap      // 19
        .word _start_I2c1_trap      // 20
        .word _start_DefaultHandler_trap // 21
        .word _start_DefaultHandler_trap // 22
        .word _start_DefaultHandler_trap // 23
        .word _start_Uart_trap      // 24
        .word _start_DefaultHandler_trap // 25
        .word _start_DefaultHandler_trap // 26
        .word _start_DefaultHandler_trap // 27
        .word _start_DefaultHandler_trap // 28
        .word _start_DefaultHandler_trap // 29
        .word _start_DefaultHandler_trap // 30
        .word _start_DefaultHandler_trap // 31
        .word _start_Timer0Ovf_trap // 32
        .word _start_Timer0Cmp_trap // 33
        .word _start_Timer1Ovf_trap // 34
        .word _start_Timer1Cmp_trap // 35
        .word _start_Timer2Ovf_trap // 36
        .word _start_Timer2Cmp_trap // 37
        .word _start_Timer3Ovf_trap // 38
        .word _start_Timer3Cmp_trap // 39
        .word _start_Timer4Ovf_trap // 40
        .word _start_Timer4Cmp_trap // 41
        .word _start_Timer5Ovf_trap // 42
        .word _start_Timer5Cmp_trap // 43
        .word _start_Timer6Ovf_trap // 44
        .word _start_Timer6Cmp_trap // 45
        .word _start_Timer7Ovf_trap // 46
        .word _start_Timer7Cmp_trap // 47
        .word _start_Timer8Ovf_trap // 48
        .word _start_Timer8Cmp_trap // 49
        .word _start_Timer9Ovf_trap // 50
        .word _start_Timer9Cmp_trap // 51
        .word _start_Timer10Ovf_trap // 52
        .word _start_Timer10Cmp_trap // 53
        .word _start_Timer11Ovf_trap // 54
        .word _start_Timer11Cmp_trap // 55
        .word _start_Timer12Ovf_trap // 56
        .word _start_Timer12Cmp_trap // 57
        .word _start_Timer13Ovf_trap // 58
        .word _start_Timer13Cmp_trap // 59
        .word _start_Timer14Ovf_trap // 60
        .word _start_Timer14Cmp_trap // 61
        .word _start_Timer15Ovf_trap // 62
        .word _start_Timer15Cmp_trap // 63
        .word _start_Ext0_trap      // 64
        .word _start_Ext1_trap      // 65
        .word _start_Ext2_trap      // 66
        .word _start_Ext3_trap      // 67

    .option pop",
);
