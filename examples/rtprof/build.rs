use riscv_target_parser::RiscvTarget;
use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    let cargo_flags = env::var("CARGO_ENCODED_RUSTFLAGS").unwrap();

    let target = RiscvTarget::build(&target, &cargo_flags).unwrap();

    // set environment variable RISCV_RT_BASE_ISA to the base ISA of the target.
    println!(
        "cargo:rustc-env=RISCV_RT_BASE_ISA={}",
        target.llvm_base_isa()
    );

    // set environment variable RISCV_RT_LLVM_ARCH_PATCH to patch LLVM bug.
    // (this env variable is temporary and will be removed after LLVM being fixed)
    println!(
        "cargo:rustc-env=RISCV_RT_LLVM_ARCH_PATCH={}",
        target.llvm_arch_patch()
    );

    println!(
        "cargo:rustc-env=RISCV_ISA={}",
        target
            .rustc_flags()
            .into_iter()
            .map(|s| s.strip_prefix("riscv").unwrap().to_owned())
            .collect::<String>()
    );

    // make sure that these env variables are not changed without notice.
    println!("cargo:rerun-if-env-changed=RISCV_ISA");
    println!("cargo:rerun-if-env-changed=RISCV_RT_BASE_ISA");
    println!("cargo:rerun-if-env-changed=RISCV_RT_LLVM_ARCH_PATCH");
}
