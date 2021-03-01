#![feature(concat_idents)]
#![feature(proc_macro_hygiene)]
#![feature(asm)]

mod shulk;

#[skyline::main(name = "shulk vision burst")]
pub fn main() {
    shulk::install();
}