#![feature(rustc_private)]


extern crate getopts;
extern crate rustc;
extern crate rustc_borrowck;
extern crate rustc_driver;
extern crate rustc_privacy;
extern crate rustc_resolve;
extern crate rustc_trans;
extern crate rustc_typeck;
extern crate syntax;


mod compiler_api;
mod custom_collections;
mod lexer;
mod render;


fn main() {
    compiler_api::main();
    println!("Hello, world!");
}
