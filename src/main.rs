#![feature(rustc_private)]
#![feature(path_ext)]


extern crate getopts;
extern crate rustc;
extern crate rustc_driver;
extern crate rustc_resolve;
extern crate rustc_trans;
extern crate syntax;


mod compiler_api;
mod custom_collections;
mod navigation;
mod html;
mod lexer;
mod render;


use std::path::Path;
use std::path::PathBuf;
use compiler_api::{
    CtxtArenas, Forest, build_session, get_main_file_path, parse_and_expand,
    assign_node_ids_and_map, analyze
};
use syntax::codemap::CodeMap;
use std::fs::File;
use std::io::Read;
use navigation::{collect_mappings, Definition, ActiveRegion};
use render::{Chunk, Wrapper, apply_wrappers};
use html::tags::{Span, A};
use std::collections::HashMap;
use path_extensions::PathExtensions;
use std::rc::Rc;
use syntax::codemap::{FileMap};
use std::hash::Hash;


mod options {
    use getopts::{Options};
    use std::path::PathBuf;
    use std::env::Args;
    use self::errors::Error;


    pub fn parse(args: Args) -> OptionsResult<Opts> {
        let parser = create_options_parser();
        let opts = try!(parser.parse(args.skip(1)));

        Ok(Opts {
            input: PathBuf::from(opts.opt_str("i").unwrap()),
            output: PathBuf::from(opts.opt_str("o").unwrap()),
            template: PathBuf::from(opts.opt_str("t").unwrap())
        })
    }


    pub type OptionsResult<T> = Result<T, Error>;


    pub struct Opts {
        pub input: PathBuf,
        pub output: PathBuf,
        pub template: PathBuf
    }


    fn create_options_parser() -> Options {
        let mut opts = Options::new();
        opts.reqopt("i", "in", "", "DIR");
        opts.reqopt("o", "out", "", "DIR");
        opts.reqopt("t", "template", "", "FILE");
        opts.optflag("h", "help", "print this help menu");
        opts
    }


    pub mod errors {
        use getopts::{Fail};
        use std::fmt;


        pub struct Error(String);


        impl From<Fail> for Error {
            fn from(err: Fail) -> Error {
                Error(err.to_string())
            }
        }


        impl<'a> From<&'a str> for Error {
            fn from(err: &str) -> Error {
                Error(err.to_string())
            }
        }

        impl fmt::Display for Error {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    }
}


fn main() {
    let options = match options::parse(std::env::args()) {
        Ok(options) => options,
        Err(err) => { println!("{}", err); return; }
    };

    println!("args");
    let (source_path, crate_type) =
        get_main_file_path(&options.input).expect("Can't find main file.");

    let sess = build_session(source_path.clone(), crate_type);
    let (id, expanded_crate) = parse_and_expand(&sess, &source_path).unwrap();

    println!("parse");
    let mut forest = Forest::new(expanded_crate);
    let arenas = CtxtArenas::new();
    let map = assign_node_ids_and_map(&sess, &mut forest);
    println!("assign node ids");
    let analysis = analyze(sess, id, map, &arenas);
    println!("analyze");

    let (active_regions, definitions) = collect_mappings(&analysis);
    println!("collect");

    let def_wrappers = definitions.into_iter().map(|x| (x.region.filename.clone(), x.to_wrapper()));
    let active_wrappers = active_regions.into_iter().map(|x| (x.region.filename.clone(), x.to_wrapper()));

    let mut wrappers_by_filename = HashMap::new();
    for (filename, wrapper) in def_wrappers.chain(active_wrappers) {
        let mut wrappers = wrappers_by_filename.entry(filename).or_insert_with(|| Vec::new());
        wrappers.push(wrapper);
    }


    let codemap = analysis.ty_cx.sess.codemap();
    for filemap in filemaps(codemap) {
        println!("\n\n>>>>>>>>>>>>>>>>>>>>>>>>>>>>> FILE: {}", &filemap.name);
        let tokens = lexer::read_tokens(filemap.clone());
        let wrappers = wrappers_by_filename.remove(&filemap.name).unwrap_or_else(|| Vec::new());
        let result = apply_wrappers(&filemap, tokens, wrappers);
        let full = render_code(&options.template, result);
        let mut result_path = options.output.join(
            &PathBuf::from(&filemap.name).relative_to(&options.input).unwrap()
        );
        result_path.set_extension("html");
        write_file(&result_path, &full)
    }
}


fn render_code<T: AsRef<str>>(template_path: &Path, data: T) -> String {
    fn render_lines(lines_count: usize) -> String {
        let mut lines_buf = String::new();
        for line in (1..lines_count + 1) {
            lines_buf.push_str(&format!("<li>{}</li>", line));
        }
        lines_buf
    }

    let lines = render_lines(data.as_ref().lines().count());
    let mut variables = HashMap::new();
    variables.insert("{{code}}", data.as_ref());
    variables.insert("{{lines}}", &lines[..]);

    render_template(template_path, &variables)
}


fn render_template<K, V>(template_path: &Path, variables: &HashMap<K, V>) -> String where K: AsRef<str> + Hash + Eq, V: AsRef<str> {
    variables.iter().fold(
        read_file(template_path),
        |template, (key, value)| template.replace(key.as_ref(), value.as_ref())
    )
}


fn read_file(path: &Path) -> String {
    use std::fs::File;

    let mut file = File::open(path).unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    buf
}


fn write_file<T: AsRef<str>>(path: &Path, data: &T) {
    use std::io::Write;
    println!("write {:?}", path);
    let mut f = File::create(path).ok().expect("create fil");
    f.write_all(data.as_ref().as_bytes()).ok().expect("write file");
}


fn filemaps(codemap: &CodeMap) -> Vec<Rc<FileMap>> {
    let mut filemaps = Vec::new();
    for fm in &*codemap.files.borrow() {
        if fm.is_real_file() && fm.src != None {
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



impl ToWrapper for ActiveRegion {
    fn to_wrapper(&self) -> Wrapper {
        let from_path = PathBuf::from(self.region.filename.clone());
        let def_path = PathBuf::from(self.def.0.clone());
        let mut path_to_def = def_path.relative_to(&from_path).unwrap();
        path_to_def.set_extension("html");
        let mut path_as_str = path_to_def.to_str().unwrap();
        if path_as_str.len() > 0 {
           path_as_str = &path_as_str[1..path_as_str.len()]
        }
        let tag = A::new().add_class("active-region").set_href(format!("{}#def-{}", path_as_str, self.def.1));
        Wrapper::new(
            Chunk::new(self.region.start, tag.render_open()),
            Chunk::new(self.region.end, tag.render_close())
        )
    }
}


mod path_extensions {
    use std::path::{Path, PathBuf, Component};


    pub trait PathExtensions {
        fn relative_to<P: ?Sized + AsRef<Path>>(&self, &P) -> Option<PathBuf>;
    }


    impl PathExtensions for Path {
        fn relative_to<P: ?Sized + AsRef<Path>>(&self, base: &P) -> Option<PathBuf> {
            let base = base.as_ref();

            if self.is_absolute() != base.is_absolute() {
                if self.is_absolute() {
                    Some(PathBuf::from(self))
                } else {
                    None
                }
            } else {
                let mut ita = self.components();
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
    }
}
