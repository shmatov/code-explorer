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
mod html;
mod lexer;
mod render;


use std::path::Path;
use compiler_api::{
    CtxtArenas, Forest, build_session, get_main_file_path, parse_and_expand,
    assign_node_ids_and_map, analyze
};
use syntax::codemap::CodeMap;
use std::fs::File;
use std::io::Read;
use goto::{collect_mappings, Definition, ActiveRegion};
use render::{Chunk, Wrapper, render};
use html::tags::Span;


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

    let (active_regions, definitions) = collect_mappings(&analysis);
    let def_wrappers = definitions.into_iter().map(|x| x.to_wrapper());
    let active_wrappers = active_regions.into_iter().map(|x| x.to_wrapper());

    let wrappers: Vec<_> = def_wrappers.chain(active_wrappers).collect();

    let codemap = CodeMap::new();
    let mut f = File::open(source_path).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    let filemap = codemap.new_filemap("".into(), s);
    let tokens = lexer::read_tokens(filemap.clone());

    let result = render(&codemap, tokens, wrappers);
    println!("{}", result);
}


trait ToWrapper {
    fn to_wrapper(&self) -> Wrapper;
}


impl ToWrapper for Definition {
    fn to_wrapper(&self) -> Wrapper {
        let tag = Span::new().add_class("definition").add_id(format!("def-{}", self.id));
        Wrapper::new(
            Chunk::new(self.region.start, tag.render_open()),
            Chunk::new(self.region.end, tag.render_close())
        )
    }
}


impl ToWrapper for ActiveRegion {
    fn to_wrapper(&self) -> Wrapper {
        let tag = Span::new().add_class("active-region");
        Wrapper::new(
            Chunk::new(self.region.start, tag.render_open()),
            Chunk::new(self.region.end, tag.render_close())
        )
    }
}
