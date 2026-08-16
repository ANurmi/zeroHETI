use riscv_target_parser::RiscvTarget;
use std::{collections::HashSet, env, fs, path::PathBuf};

fn add_linker_script() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    if cfg!(feature = "rt") {
        // Put the linker script somewhere the linker can find it.
        fs::write(out_dir.join("memory.x"), include_bytes!("memory.x")).unwrap();
        println!("cargo:rustc-link-search={}", out_dir.display());
        println!("cargo:rerun-if-changed=memory.x");
    }
}

/// Parse the target RISC-V architecture and returns its bit width and the
/// extension set
///
/// Returns a char set such as ['i', 'e'] based on which extensions are active
/// right now.
fn parse_extensions(target: &str, cargo_flags: &str) -> HashSet<char> {
    // isolate bit width and extensions from the rest of the target information
    let arch = target
        .trim_start_matches("riscv")
        .split('-')
        .next()
        .unwrap();

    let mut extensions: HashSet<char> = arch.chars().skip_while(|c| c.is_ascii_digit()).collect();
    // expand the 'g' shorthand extension
    if extensions.contains(&'g') {
        extensions.insert('i');
        extensions.insert('m');
        extensions.insert('a');
        extensions.insert('f');
        extensions.insert('d');
    }

    let cargo_flags = cargo_flags
        .split(0x1fu8 as char)
        .filter(|arg| !arg.is_empty());

    cargo_flags
        .filter(|k| k.starts_with("target-feature="))
        .flat_map(|str| {
            let flags = str.split('=').collect::<Vec<&str>>()[1];
            flags.split(',')
        })
        .for_each(|feature| {
            let chars = feature.chars().collect::<Vec<char>>();
            match chars[0] {
                '+' => {
                    extensions.insert(chars[1]);
                }
                '-' => {
                    extensions.remove(&chars[1]);
                }
                _ => {
                    panic!("Unsupported target feature operation");
                }
            }
        });

    extensions
}

fn main() {
    add_linker_script();

    let target = env::var("TARGET").unwrap();
    let cargo_flags = env::var("CARGO_ENCODED_RUSTFLAGS").unwrap();

    // Set configuration flags depending on the target
    if target.starts_with("riscv") {
        let extensions = parse_extensions(&target, &cargo_flags);

        // Collect extensions to an environment variable that can be printed from the program
        println!(
            "cargo::rustc-env=RISCV_EXTS={}",
            extensions
                .iter()
                .fold(String::new(), |acc, x| acc + &x.to_string())
        );
    }
    // set environment variable RISCV_RT_BASE_ISA to the base ISA of the target.
    let target = RiscvTarget::build(&target, &cargo_flags).unwrap();
    println!(
        "cargo:rustc-env=RISCV_RT_BASE_ISA={}",
        target.llvm_base_isa()
    );

    println!("cargo:rerun-if-env-changed=RISCV_RT_BASE_ISA");
    println!("cargo:rerun-if-changed=build.rs");
}
