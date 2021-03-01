#![feature(concat_idents)]
#![feature(proc_macro_hygiene)]
#![feature(asm)]

mod lucina;

#[skyline::main(name = "yu_narukami")]
pub fn main() {
    lucina::install();
}