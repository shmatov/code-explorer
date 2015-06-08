use std::collections::HashMap;

use compiler_api::CrateAnalysis;
use lexer::{Interval, to_interval};

use syntax::ast::{DefId, NodeId};
use rustc::middle::def::Def;
use rustc::middle::def::PathResolution;


pub fn build_goto_table(crate_analysis: &CrateAnalysis) -> HashMap<Interval, Interval> {
    let codemap = crate_analysis.ty_cx.sess.codemap();
    let def_map = crate_analysis.ty_cx.def_map.borrow();

    def_map.iter()
        .map(|(&node_id, path)| (
            crate_analysis.ty_cx.map.opt_span(node_id)
                .map(|span| to_interval(span, codemap.span_to_filename(span))),
            get_node_id_by_path(path).and_then(|id| crate_analysis.ty_cx.map.opt_span(id))
                .map(|span| to_interval(span, codemap.span_to_filename(span)))
        ))
        .filter_map(|spans| match spans {
            (Some(node_span), Some(def_span)) => Some((node_span, def_span)),
            _ => None
        })
        .filter(|&(ref node_span, ref def_span)| node_span != def_span)
        .collect()
}



fn get_node_id_by_path(path: &PathResolution) -> Option<NodeId> {
    get_def_id(path).and_then(|def_id| get_node_id(def_id))
}



fn get_def_id(path: &PathResolution) -> Option<DefId> {
    match path.full_def() {
        Def::DefPrimTy(_) | Def::DefSelfTy(..) => None,
        _ => Some(path.def_id())
    }
}


fn get_node_id(def_id: DefId) -> Option<NodeId> {
    use syntax::ast::LOCAL_CRATE;

    if def_id.krate == LOCAL_CRATE {
        Some(def_id.node)
    } else {
        None
    }
}
