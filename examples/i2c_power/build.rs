use riscv_target_parser::RiscvTarget;
use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    let cargo_flags = env::var("CARGO_ENCODED_RUSTFLAGS").unwrap();

    if let Ok(target) = RiscvTarget::build(&target, &cargo_flags) {
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
    }
}
