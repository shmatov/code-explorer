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
mod goto;
mod lexer;
mod render;


use std::path::Path;
use compiler_api::{
    CtxtArenas, Forest, build_session, get_main_file_path, parse_and_expand,
    assign_node_ids_and_map, analyze
};
use goto::build_goto_table;


fn main() {
    let crate_path = std::env::args().nth(1).unwrap();
    let (source_path, crate_type) =
        get_main_file_path(&Path::new(&crate_path)).expect("Can't find main file.");

    let sess = build_session(source_path.clone(), crate_type);
    let (id, expanded_crate) = parse_and_expand(&sess, &source_path).unwrap();

    let mut forest = Forest::new(expanded_crate);
    let arenas = CtxtArenas::new();
    let map = assign_node_ids_and_map(&sess, &mut forest);
    let analysis = analyze(sess, id, map, &arenas);

    let table = build_goto_table(&analysis);
    println!("{:#?}", table);
}
