use std::path::{Path, PathBuf};

pub use syntax::ast::Crate;
pub use syntax::ast_map::{Forest, Map};
pub use rustc_driver::driver::assign_node_ids_and_map;
pub use rustc::middle::ty::{CrateAnalysis, CtxtArenas};

use rustc::session::config::{CrateType, Input, build_configuration};
use rustc::session::Session;

use rustc_driver;
use rustc_resolve;


pub fn parse_and_expand(sess: &Session, source_path: &Path) -> Option<(String, Crate)> {
    use rustc_trans::back::link;
    use rustc_driver::driver::{
        phase_1_parse_input,
        phase_2_configure_and_expand
    };

    let cfg = build_configuration(&sess);

    let input = &Input::File(source_path.into());
    let krate = phase_1_parse_input(sess, cfg, input);

    let id = link::find_crate_name(
        Some(sess), &krate.attrs, input
    );

    phase_2_configure_and_expand(
        sess, krate, &id[..], None
    ).map(|krate| (id, krate))
}


pub fn analyze<'ast>(sess: Session, crate_id: String, ast_map: Map<'ast>, arenas: &'ast CtxtArenas<'ast>) -> CrateAnalysis<'ast> {
    use rustc_driver::driver::phase_3_run_analysis_passes;

    phase_3_run_analysis_passes(
        sess, ast_map, arenas, crate_id, rustc_resolve::MakeGlobMap::Yes
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
    use rustc::session::config::{basic_options, UnstableFeatures};

    let descriptions = rustc_driver::diagnostics_registry();

    let mut sopts = basic_options();
    sopts.unstable_features = UnstableFeatures::Default;
    sopts.crate_types.push(crate_type);
    sopts.maybe_sysroot = Some(PathBuf::from("/usr/local"));

    session::build_session(
        sopts, Some(input_file_path), descriptions
    )
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
