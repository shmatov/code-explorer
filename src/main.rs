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
use html::tags::{Span, A};
use std::collections::HashMap;
use getopts::{Options, Matches};


fn create_options_parser() -> Options {
    let mut opts = Options::new();
    opts.reqopt("i", "in", "", "DIR");
    opts.reqopt("o", "out", "", "DIR");
    opts.reqopt("t", "template", "", "FILE");
    opts.optflag("h", "help", "print this help menu");
    opts
}


fn parse_options() -> Matches {
    use std::env;

    let parser = create_options_parser();
    match parser.parse(env::args().skip(1)) {
        Ok(matches) => matches,
        Err(err) => panic!(err.to_string())
    }
}


fn main() {
    let options = parse_options();
    let crate_path = options.opt_str("i").unwrap();
    let result_path = options.opt_str("o").unwrap();
    let template_path = options.opt_str("t").unwrap();


    let (source_path, crate_type) =
        get_main_file_path(&Path::new(&crate_path)).expect("Can't find main file.");

    let sess = build_session(source_path.clone(), crate_type);
    let (id, expanded_crate) = parse_and_expand(&sess, &source_path).unwrap();

    let mut forest = Forest::new(expanded_crate);
    let arenas = CtxtArenas::new();
    let map = assign_node_ids_and_map(&sess, &mut forest);
    let analysis = analyze(sess, id, map, &arenas);

    let (active_regions, definitions) = collect_mappings(&analysis);

    let def_wrappers = definitions.into_iter().map(|x| (x.region.filename.clone(), x.to_wrapper()));
    let active_wrappers = active_regions.into_iter().map(|x| (x.region.filename.clone(), x.to_wrapper()));

    let mut wrappers_by_filename = HashMap::new();
    for (filename, wrapper) in def_wrappers.chain(active_wrappers) {
        let mut wrappers = wrappers_by_filename.entry(filename).or_insert_with(|| Vec::new());
        wrappers.push(wrapper);
    }

    let output = PathBuf::from(result_path);

    let codemap = analysis.ty_cx.sess.codemap();
    for filemap in filemaps(codemap) {
        println!("\n\n>>>>>>>>>>>>>>>>>>>>>>>>>>>>> FILE: {}", &filemap.name);
        let tokens = lexer::read_tokens(filemap.clone());
        let wrappers = wrappers_by_filename.remove(&filemap.name).unwrap_or_else(|| Vec::new());
        let result = render(&filemap, tokens, wrappers);
        let full = render_file(&template_path[..], &result[..]);
        let mut result_path = output.join(
            &path_relative_from(&PathBuf::from(&filemap.name), &PathBuf::from(&crate_path)).unwrap()
        );
        result_path.set_extension("html");
        write_file(&result_path, full)
    }
}


fn write_file(path: &Path, data: String) {
    use std::io::Write;
    println!("write {:?}", path);
    let mut f = File::create(path).ok().expect("create fil");
    f.write_all(data.as_bytes()).ok().expect("write file");
}

fn render_file(path: &str, data: &str) -> String {
    let mut lines_buf = String::new();
    for line in (1..data.lines().count() + 1) {
        lines_buf.push_str(&format!("<li>{}</li>", line));
    }

    let mut f = File::open(path).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s.replace("{{code}}", data).replace("{{lines}}", &lines_buf)
}


use std::rc::Rc;
use syntax::codemap::{FileMap};

fn filemaps(codemap: &CodeMap) -> Vec<Rc<FileMap>> {
    let mut filemaps = Vec::new();
    for fm in &*codemap.files.borrow() {
        if fm.is_real_file() {
            filemaps.push(fm.clone());
        }
    }
    filemaps
}


trait ToWrapper {
    fn to_wrapper(&self) -> Wrapper;
}


impl ToWrapper for Definition {
    fn to_wrapper(&self) -> Wrapper {
        let tag = Span::new()
            .add_class("definition")
            .add_id(format!("def-{}", self.id))
            .set_name(format!("def-{}", self.id));
        Wrapper::new(
            Chunk::new(self.region.start, tag.render_open()),
            Chunk::new(self.region.end, tag.render_close())
        )
    }
}


use std::path::PathBuf;

impl ToWrapper for ActiveRegion {
    fn to_wrapper(&self) -> Wrapper {
        let from_path = PathBuf::from(self.region.filename.clone());
        let def_path = PathBuf::from(self.def.0.clone());
        let mut path_to_def = path_relative_from(&def_path, &from_path).unwrap();
        path_to_def.set_extension("html");
        let tag = A::new().add_class("active-region").set_href(format!("{}#def-{}", path_to_def.to_str().unwrap(), self.def.1));
        Wrapper::new(
            Chunk::new(self.region.start, tag.render_open()),
            Chunk::new(self.region.end, tag.render_close())
        )
    }
}

fn path_relative_from(path: &Path, base: &Path) -> Option<PathBuf> {
    use std::path::Component;

    if path.is_absolute() != base.is_absolute() {
        if path.is_absolute() {
            Some(PathBuf::from(path))
        } else {
            None
        }
    } else {
        let mut ita = path.components();
        let mut itb = base.components();
        let mut comps: Vec<Component> = vec![];
        loop {
            match (ita.next(), itb.next()) {
                (None, None) => break,
                (Some(a), None) => {
                    comps.push(a);
                    comps.extend(ita.by_ref());
                    break;
                }
                (None, _) => comps.push(Component::ParentDir),
                (Some(a), Some(b)) if comps.is_empty() && a == b => (),
                (Some(a), Some(b)) if b == Component::CurDir => comps.push(a),
                (Some(_), Some(b)) if b == Component::ParentDir => return None,
                (Some(a), Some(_)) => {
                    comps.push(Component::ParentDir);
                    for _ in itb {
                        comps.push(Component::ParentDir);
                    }
                    comps.push(a);
                    comps.extend(ita.by_ref());
                    break;
                }
            }
        }
        Some(comps.iter().map(|c| c.as_os_str()).collect())
    }
}
