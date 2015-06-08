use std::path::{Path, PathBuf};

use rustc::middle::ty::CtxtArenas;
use rustc::session::config::{CrateType, Input, build_configuration};
use rustc::session::{self, Session};
use rustc_driver;
use rustc_resolve;
use syntax::{ast, ast_map};


pub fn analyze_crate(crate_path: &Path) -> analysis::Analysis {
    let (source_path, crate_type) =
        get_main_file_path(crate_path).expect("Can't find main file.");
    compile(source_path, crate_type)
}


fn get_main_file_path(crate_path: &Path) -> Option<(PathBuf, CrateType)> {
    use std::fs::PathExt;

    vec![
        (crate_path.join("src/main.rs"), CrateType::CrateTypeExecutable),
        (crate_path.join("src/lib.rs"), CrateType::CrateTypeDylib)
    ].into_iter().inspect(|x| println!("{:?}", x)).find(|&(ref path, _)| path.is_file())
}


fn compile(input_file_path: PathBuf, crate_type: CrateType) -> analysis::Analysis {
    let sess = build_session(input_file_path.clone(), crate_type);
    let cfg = build_configuration(&sess);

    compile_input(sess, cfg, input_file_path)
}


fn build_session(input_file_path: PathBuf, crate_type: CrateType) -> Session {
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


fn compile_input(sess: Session, cfg: ast::CrateConfig, input_file_path: PathBuf) -> analysis::Analysis {
    use rustc_trans::back::link;
    use rustc_driver::driver::{
        phase_1_parse_input,
        phase_2_configure_and_expand,
        assign_node_ids_and_map,
        phase_3_run_analysis_passes,
    };


    let input = &Input::File(input_file_path);
    let krate = phase_1_parse_input(&sess, cfg, input);

    let id = link::find_crate_name(
        Some(&sess), &krate.attrs, input
    );
    let expanded_crate = phase_2_configure_and_expand(
        &sess, krate, &id[..], None
    ).unwrap();

    let mut forest = ast_map::Forest::new(expanded_crate);
    let arenas = CtxtArenas::new();
    let ast_map = assign_node_ids_and_map(&sess, &mut forest);

    let analysis = phase_3_run_analysis_passes(
        sess, ast_map, &arenas, id, rustc_resolve::MakeGlobMap::Yes
    );
    println!("Def map: {:?}", analysis.ty_cx.def_map);
    analysis::Analysis { def_map: analysis.ty_cx.def_map }
}


mod analysis {
    use rustc::middle::def::DefMap;

    pub struct Analysis {
        pub def_map: DefMap
    }
}
