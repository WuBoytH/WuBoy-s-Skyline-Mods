#![feature(concat_idents)]
#![feature(proc_macro_hygiene)]

mod shulk;
mod custom;

#[skyline::main(name = "the_bor_patch")]
pub fn main() {
    shulk::install();
    custom::install();
}