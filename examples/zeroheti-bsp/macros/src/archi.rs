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
/// This is the caller-saved set of the *standard* ILP32E ABI, which is what
/// rustc generates (incl. `compiler_builtins`). The nested interrupt prologue
/// must stack all of these registers: compiler generated code may hold a live
/// value in any of them (e.g. `__udivdi3` clobbers `a4`/`x14`) when an
/// interrupt fires. See issue #106.
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
/// This is the caller-saved set of the *standard* ILP32 ABI, which is what
/// rustc generates. The nested interrupt prologue must stack all of these
/// registers. See issue #106.
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
///
/// This reduced set is what the hardware PCS save mechanism stacks and what the
/// CLIC fast-interrupt documentation prescribes. It is NOT what rustc's
/// standard ILP32E codegen guarantees to preserve (`x6`, `x7` and `x14` are
/// caller-saved in the standard ABI), so it is kept here only as a reference:
/// the software trampoline must use the more pessimistic [`CALLER_SAVE_RVE`]
/// set instead. See issue #106.
#[allow(dead_code)]
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
