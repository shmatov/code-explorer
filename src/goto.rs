use compiler_api::CrateAnalysis;
pub use syntax::ast::NodeId;
use std::collections::HashMap;


pub fn collect_mappings(crate_analysis: &CrateAnalysis) -> (Vec<ActiveRegion>, Vec<Definition>) {
    let ty_cx = &crate_analysis.ty_cx;
    let codemap = ty_cx.sess.codemap();
    let def_map = ty_cx.def_map.borrow();

    let mut definitions_generator = UniqRegionRegistry::new(
        |region, id| Definition { region: region, id: id}
    );

    let mappings = def_map.iter()
        .map(|(&node_id, path)| (
            conversions::node_id_to_span(&ty_cx.map, node_id)
                .and_then(|span| conversions::span_to_region(codemap, span)),
            conversions::path_resolution_to_span(&ty_cx.map, path)
                .and_then(|span| conversions::span_to_region(codemap, span))
        ))
        .filter_map(has_both)
        .filter(|&(ref a, ref b)| a != b);

    let mut active_regions = Vec::new();
    for (active_region, def_region) in mappings {
        let def_id = definitions_generator.get_or_register(def_region);
        active_regions.push(ActiveRegion { definition_id: def_id, region: active_region});
    }
    (active_regions, definitions_generator.generate())
}


struct UniqRegionRegistry<'a, T> {
    region_to_id: HashMap<Region, u32>,
    id: u32,
    constructor: Box<Fn(Region, u32) -> T + 'a>
}


impl<'a, T> UniqRegionRegistry<'a, T> {
    pub fn new<F: Fn(Region, u32) -> T + 'a>(constructor: F) -> UniqRegionRegistry<'a, T> {
        UniqRegionRegistry {
            region_to_id: HashMap::new(),
            id: 0,
            constructor: Box::new(constructor)
        }
    }

    pub fn get_or_register(&mut self, region: Region) -> u32 {
        if let Some(&id) = self.region_to_id.get(&region) {
            return id;
        }
        self.region_to_id.insert(region, self.id);
        self.id += 1;
        self.id
    }

    pub fn generate(self) -> Vec<T> {
        let mut items = Vec::new();
        for (region, id) in self.region_to_id {
            items.push((self.constructor)(region, id));
        }
        items
    }
}


fn has_both<T>(tuple: (Option<T>, Option<T>)) -> Option<(T, T)> {
    match tuple {
        (Some(a), Some(b)) => Some((a, b)),
        _ => None
    }
}


#[derive(Clone)]
pub struct Definition {
    pub id: u32,
    pub region: Region
}


pub struct ActiveRegion {
    pub definition_id: u32,
    pub region: Region
}


#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Region {
    pub filename: String,
    pub start: usize, //inclusive character position
    pub end: usize, //inclusive character position
}


mod conversions {
    use super::Region;

    use rustc::middle::def::{Def, PathResolution};
    use syntax::ast::{DefId, NodeId};
    use syntax::ast_map::Map;
    use syntax::codemap::{Span, CodeMap};


    pub fn span_to_region(codemap: &CodeMap, span: Span) -> Option<Region> {
        use syntax::codemap::{Pos, DUMMY_SP};
        if span == DUMMY_SP {
            return None;
        }
        Some(Region {
            start: codemap.bytepos_to_file_charpos(span.lo).to_usize(),
            end: codemap.bytepos_to_file_charpos(span.hi).to_usize() - 1,
            filename: codemap.span_to_filename(span)
        })
    }


    pub fn node_id_to_span<'ast>(map: &Map<'ast>, node_id: NodeId) -> Option<Span> {
        map.opt_span(node_id)
    }


    pub fn path_resolution_to_span<'ast>(map: &Map<'ast>, path: &PathResolution) -> Option<Span> {
        path_resolution_to_node_id(path).and_then(|node_id| node_id_to_span(map, node_id))
    }


    fn path_resolution_to_node_id(path: &PathResolution) -> Option<NodeId> {
        path_resolution_to_def_id(path).and_then(|def_id| def_id_to_node_id(def_id))
    }


    fn path_resolution_to_def_id(path: &PathResolution) -> Option<DefId> {
        match path.full_def() {
            Def::DefPrimTy(_) | Def::DefSelfTy(..) => None,
            _ => Some(path.def_id())
        }
    }


    fn def_id_to_node_id(def_id: DefId) -> Option<NodeId> {
        use syntax::ast::LOCAL_CRATE;

        if def_id.krate == LOCAL_CRATE {
            Some(def_id.node)
        } else {
            None
        }
    }
}
