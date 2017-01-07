extern crate syn;

use fsm_def::*;
use petgraph::*;

use petgraph::dot::{
    Dot,
};

use petgraph::algo::{
    DfsSpace,
};

use petgraph::visit::*;

use std::collections::HashMap;


#[derive(Debug)]
struct NodeData {
    state: String,
    region: usize
}

use std::fmt;
impl fmt::Display for NodeData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "state: {}, region: {}", self.state, self.region)
    }
}

pub fn create_regions(transitions: &Vec<TransitionEntry>, initial_states: &Vec<syn::Ty>, submachines: &Vec<syn::Ty>, interrupt_states: &Vec<FsmInterruptState>) -> Vec<FsmRegion> {
    
    let mut gr = Graph::new();
    let mut nodes = HashMap::new();

    let orphan_region = 255;

    for initial_state in initial_states {
        let s = ty_to_string(initial_state);

        let n = gr.add_node(NodeData { state: s.clone(), region: orphan_region });
        nodes.insert(s, n);
    }

    for transition in transitions {
        let (src, target) = {
            let mut get_node = |ty| {
                let s = ty_to_string(ty);

                let mut node = None;
                                
                if let Some(n) = nodes.get(&s) {
                    node = Some(*n);
                }
                
                if node.is_none() {
                    let n = gr.add_node(NodeData { state: s.clone(), region: orphan_region });
                    nodes.insert(s, n);
                    node = Some(n)
                }

                node.unwrap()
            };

            (get_node(&transition.source_state), get_node(&transition.target_state))
        };

        gr.add_edge(src, target, 0);
    }

    let mut regions = Vec::new();
    let mut region_id = 0;
    for initial_state in initial_states {
        let s = ty_to_string(initial_state);

        let node = nodes.get(&s).expect(&format!("Missing initial state {} in graph?", &s));

        let mut dfs = Dfs::new(&gr, *node);
        while let Some(nx) = dfs.next(&gr) {
            gr[nx].region = region_id;
        }        

        regions.push(FsmRegion {
            submachines: Vec::new(),
            id: region_id,
            transitions: Vec::new(),
            initial_state_ty: initial_state.clone(),
            interrupt_states: Vec::new()
        });

        region_id += 1;
    }

    if initial_states.len() != region_id {
        panic!("Mismatch between the length of the state tuple and number of detected regions: {} state tuple length, {} regions", initial_states.len(), region_id);
    }
    
    for node in gr.raw_nodes() {
        if node.weight.region == orphan_region {
            panic!("Unreachable state: {}", node.weight.state);
        }
    }

    for transition in transitions {
        let s = ty_to_string(&transition.source_state);
        let node = *nodes.get(&s).unwrap();
        let r_id = gr[node].region;
        let ref mut r = regions[r_id];
        r.transitions.push(transition.clone());
    }

    for mut region in &mut regions {
        let states = region.get_all_states().clone();
        for s in &states {
            if submachines.contains(s) {
                region.submachines.push(s.clone());
            }
            let region_interrupted_states = interrupt_states.iter().filter(|x| &x.interrupt_state_ty == s);
            for interrupted_state in region_interrupted_states {
                region.interrupt_states.push(interrupted_state.clone());
            }
        }
    }
    
    regions
}
