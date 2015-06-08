#![feature(rustc_private)]
#![feature(path_ext)]


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


use std::path::Path;


fn main() {
    compiler_api::analyze_crate(&Path::new("/tmp/hello_world"));
    println!("Hello, world!");
}
