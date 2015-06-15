use std::path::{Path, PathBuf};

pub use syntax::ast::Crate;
pub use syntax::ast_map::{Forest, Map};
pub use rustc_driver::driver::assign_node_ids_and_map;
pub use rustc::middle::ty::{CrateAnalysis, CtxtArenas};

use rustc::session::config::{CrateType, Input, build_configuration, self};
use rustc::session::Session;

use rustc_driver;
use rustc_resolve;
use getopts;


pub fn parse_and_expand(sess: &Session, source_path: &Path) -> Option<(String, Crate)> {
    use rustc_trans::back::link;
    use rustc_driver::driver::{
        phase_1_parse_input,
        phase_2_configure_and_expand
    };

    let cfg = build_configuration(&sess);

    let input = &Input::File(source_path.into());
    println!("phase_1_parse_input");
    let krate = phase_1_parse_input(sess, cfg, input);

    println!("find_crate_name");
    let id = link::find_crate_name(
        Some(sess), &krate.attrs, input
    );

    println!("phase_2_configure_and_expand");
    phase_2_configure_and_expand(
        sess, krate, &id[..], None
    ).map(|krate| (id, krate))
}


pub fn analyze<'ast>(sess: Session, crate_id: String, ast_map: Map<'ast>, arenas: &'ast CtxtArenas<'ast>) -> CrateAnalysis<'ast> {
    use rustc_driver::driver::phase_3_run_analysis_passes;

    phase_3_run_analysis_passes(
        sess, ast_map, arenas, crate_id, rustc_resolve::MakeGlobMap::No
    )
}


pub fn get_main_file_path(crate_path: &Path) -> Option<(PathBuf, CrateType)> {
    use std::fs::PathExt;

    vec![
        (crate_path.join("main.rs"), CrateType::CrateTypeExecutable),
        (crate_path.join("lib.rs"), CrateType::CrateTypeDylib)
    ].into_iter().find(|&(ref path, _)| path.is_file())
}


pub fn build_session(input_file_path: PathBuf, crate_type: CrateType) -> Session {
    use rustc::session;
    use rustc::session::config::{basic_options, UnstableFeatures, self};
    use rustc::session::config::DebugInfoLevel;
    use rustc_driver::CompilerCalls;

    let descriptions = rustc_driver::diagnostics_registry();

    //let mut sopts = basic_options();
    //sopts.unstable_features = UnstableFeatures::Default;
    //sopts.crate_types.push(crate_type);
    //sopts.maybe_sysroot = Some(PathBuf::from("/usr/local"));
    //sopts.debug_assertions = true;
    //sopts.debuginfo = DebugInfoLevel::FullDebugInfo;


    let mut callbacks = rustc_driver::RustcDefaultCalls;

    let command = format!("rustc --sysroot /usr/local --crate-type lib {}", input_file_path.to_str().unwrap());
    let args: Vec<_> = command.split(" ").map(|x| x.to_string()).collect();

    let matches = match rustc_driver::handle_options(args) {
        Some(matches) => matches,
        None => panic!("cant handle options")
    };

    let sopts = config::build_session_options(&matches);

    //let (odir, ofile) = make_output = {
    //    let odir = &matches.opt_str("out-dir").map(|o| PathBuf::from(&o));
    //    let ofile = &matches.opt_str("o").map(|o| PathBuf::from(&o));
    //    (odir, ofile)
    //};

    //let (odir, ofile) = make_output(&matches);
    //let (input, input_file_path) = match make_input(&matches.free) {
    //    Some((input, input_file_path)) => callbacks.some_input(input, input_file_path),
    //    None => match callbacks.no_input(&matches, &sopts, &odir, &ofile, &descriptions) {
    //        Some((input, input_file_path)) => (input, input_file_path),
    //        None => panic!("cant find input")
    //    }
    //};

    //let sess = session::build_session(sopts, input_file_path, descriptions);

    session::build_session(
        sopts, Some(input_file_path), descriptions
    )
}





fn make_input(free_matches: &[String]) -> Option<(config::Input, Option<PathBuf>)> {
    use std::io::{self, Read};

    if free_matches.len() == 1 {
        let ifile = &free_matches[0][..];
        if ifile == "-" {
            let mut src = String::new();
            io::stdin().read_to_string(&mut src).unwrap();
            Some((config::Input::Str(src), None))
        } else {
            Some((config::Input::File(PathBuf::from(ifile)), Some(PathBuf::from(ifile))))
        }
    } else {
        None
    }
}


#[cfg(test)]
mod tests {
    use std::path::Path;
    use super::{
        CtxtArenas, Forest, build_session, get_main_file_path, parse_and_expand,
        assign_node_ids_and_map, analyze
    };

    #[test]
    fn test() {
        let crate_path = Path::new("/tmp/hello_world/");
        let (source_path, crate_type) =
            get_main_file_path(crate_path).expect("Can't find main file.");

        let sess = build_session(source_path.clone(), crate_type);
        let (id, expanded_crate) = parse_and_expand(&sess, &source_path).unwrap();

        let mut forest = Forest::new(expanded_crate);
        let arenas = CtxtArenas::new();
        let map = assign_node_ids_and_map(&sess, &mut forest);
        let analysis = analyze(sess, id, map, &arenas);
    }
}
