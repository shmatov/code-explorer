use std::path::PathBuf;

use rustc::middle::ty;
use rustc::session::{self, config};
use rustc_driver;
use rustc_resolve;
use syntax::{ast, ast_map};


pub fn main() {
    compile(PathBuf::from("/tmp/example.rs"));
}


fn compile(input_file_path: PathBuf) {
    let input = config::Input::File(input_file_path.clone());
    let descriptions = rustc_driver::diagnostics_registry();

    let mut sopts = config::basic_options();
    sopts.unstable_features = config::UnstableFeatures::Default;
    sopts.maybe_sysroot = Some(PathBuf::from("/usr/local"));

    let sess = session::build_session(sopts, Some(input_file_path), descriptions);
    let cfg = config::build_configuration(&sess);

    compile_input(sess, cfg, &input);
}


pub fn compile_input(sess: session::Session, cfg: ast::CrateConfig, input: &config::Input) {
    use rustc_trans::back::link;
    use rustc_driver::driver::{
        phase_1_parse_input,
        phase_2_configure_and_expand,
        assign_node_ids_and_map,
        phase_3_run_analysis_passes,
    };

    let krate = phase_1_parse_input(&sess, cfg, input);

    let id = link::find_crate_name(
        Some(&sess), &krate.attrs, input
    );
    let expanded_crate = phase_2_configure_and_expand(
        &sess, krate, &id[..], None
    ).unwrap();


    let mut forest = ast_map::Forest::new(expanded_crate);
    let arenas = ty::CtxtArenas::new();
    let ast_map = assign_node_ids_and_map(&sess, &mut forest);

    let analysis = phase_3_run_analysis_passes(
        sess, ast_map, &arenas, id, rustc_resolve::MakeGlobMap::Yes
    );
    println!("Def map: {:?}", analysis.ty_cx.def_map);
}
