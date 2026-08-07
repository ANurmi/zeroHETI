use constcat::concat_slices;

/// Software calling convention, implying register set
pub(crate) enum Abi {
    // Full set (32x 32-bit registers)
    Ilp32,
    // Reduced set (16x 32-bit registers), targeting RVE
    Ilp32e,
}

/// RISC-V standard ABI on RVE/ILP32E
///
/// These registers are saved by the interrupt prologue and should not be
/// preserved across compiler generated calls.
#[rustfmt::skip]
pub(crate) const CALLER_SAVE_RVE: &[&str] = &[
    // `ra`: return address, stores the address to return to after a function call or interrupt.
    "x1",
    // `t0`..=`t2`: temporary/link register
    "x5", "x6", "x7",
    // `a0`..=`a1`: Argument/return value
    "x10", "x11",
    // `a2`..=`a5`: Argument
    "x12", "x13", "x14", "x15",
];

/// RISC-V standard ABI on RVI/ILP32
///
/// These registers are saved by the interrupt prologue and should not be
/// preserved across compiler generated calls.
#[rustfmt::skip]
pub(crate) const CALLER_SAVE_RVI: &[&str] = concat_slices!([&str]:
    CALLER_SAVE_RVE,
    &[
        // `a6`..=`t7`
        "x16", "x17",
        // `t3`..=`t6`
        "x28", "x29", "x30", "x31",
    ]
);

/// RISC-V EABI
///
/// These registers are saved by the interrupt prologue and should not be
/// preserved across compiler generated calls.
///
/// [RISC-V EABI](https://github.com/riscvarchive/riscv-eabi-spec/)
#[rustfmt::skip]
pub(crate) const CALLER_SAVE_EABI: &[&str] = &[
    // `ra`: return address, stores the address to return to after a function call or interrupt.
    "x1",
    // `t0`: temporary/link register
    "x5",
    // `a0`..=`a1`: Argument/return value
    "x10", "x11",
    // `a2`..=`a3`: Argument
    "x12", "x13",
    // `t1`: Temporary (`a5` in RISC-V standard ABI)
    "x15",
];

mod _private {
    use constcat::concat_slices;

    /// Callee-save registers are saved on demand by the compiler. This list is not
    /// useful to code but kept here in source as reference.
    #[allow(dead_code)]
    #[rustfmt::skip]
    const CALLEE_SAVE_EABI_RVE: &[&str] = &[
        // `sp`: stack pointer
        "x2",
        // `s3`..=`s4`: saved register (`t1`..=`t2` in RISC-V ABI)
        "x6", "x7",
        // `s0`/`fp`: saved register/frame pointer
        "x8",
        // `s1`: saved register
        "x9",
        // `s2`: saved register (`a4` in RISC-V ABI)
        "x14",
    ];

    /// Callee-save registers are saved on demand by the compiler. This list is not
    /// useful to code but kept here in source as reference.
    /// [RISC-V EABI](https://github.com/riscvarchive/riscv-eabi-spec/)
    #[allow(dead_code)]
    #[rustfmt::skip]
    const CALLEE_SAVE_EABI_RVI: &[&str] = concat_slices!([&str]:
        CALLEE_SAVE_EABI_RVE,
        &[
            // `x16..=x31`: `s5-s20` saved registers (`a6-a7`, s2-s11`, `t3-t6` in RISC-V ABI)
            "x16", "x17", "x18", "x19", "x20", "x21", "x22", "x23",
            "x24", "x25", "x26", "x27", "x28", "x29", "x30", "x31",
        ]
    );
}
