//! Proc-macros used by `atalanta_bsp`
mod archi;
mod trampoline;
mod validate;

use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::archi::Abi;

// We need variations for ilp32/ilp32e and pcs/non-pcs

/// Sa. [crate::trampoline::nested_interrupt]
#[proc_macro_attribute]
pub fn nested_interrupt_ilp32(args: TokenStream, input: TokenStream) -> TokenStream {
    trampoline::nested_interrupt(args, input, Abi::Ilp32)
}

#[proc_macro_attribute]
pub fn nested_interrupt_ilp32e(args: TokenStream, input: TokenStream) -> TokenStream {
    trampoline::nested_interrupt(args, input, Abi::Ilp32e)
}

/// Sa. [crate::trampoline::generate_pcs_trap_entry]
#[proc_macro]
pub fn generate_pcs_trap_entry(input: TokenStream) -> TokenStream {
    let interrupt = parse_macro_input!(input as syn::Ident);
    trampoline::generate_pcs_trap_entry(&interrupt.to_string()).into()
}

/// Sa. [crate::trampoline::generate_nested_trap_entry]
#[proc_macro]
pub fn generate_nested_trap_entry_ilp32(input: TokenStream) -> TokenStream {
    let interrupt = parse_macro_input!(input as syn::Ident);
    trampoline::generate_nested_trap_entry(&interrupt.to_string(), Abi::Ilp32).into()
}

/// Sa. [crate::trampoline::generate_nested_trap_entry]
#[proc_macro]
pub fn generate_nested_trap_entry_ilp32e(input: TokenStream) -> TokenStream {
    let interrupt = parse_macro_input!(input as syn::Ident);
    trampoline::generate_nested_trap_entry(&interrupt.to_string(), Abi::Ilp32e).into()
}

/// Sa. [crate::trampoline::generate_continue_nested_trap_impl]
#[proc_macro]
pub fn generate_continue_nested_trap_ilp32(_input: TokenStream) -> TokenStream {
    trampoline::generate_continue_nested_trap_impl(Abi::Ilp32)
}

/// Sa. [crate::trampoline::generate_continue_nested_trap_impl]
#[proc_macro]
pub fn generate_continue_nested_trap_ilp32e(_input: TokenStream) -> TokenStream {
    trampoline::generate_continue_nested_trap_impl(Abi::Ilp32e)
}
