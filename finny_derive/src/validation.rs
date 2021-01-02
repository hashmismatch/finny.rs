use std::collections::HashMap;

use petgraph::{Graph, graph::NodeIndex, visit::Dfs};
use syn::spanned::Spanned;

use crate::parse::FsmDeclarations;

#[derive(Debug)]
struct TypeNode {
    state: syn::Type,
    region: Option<usize>
}

pub fn create_regions(decl: &FsmDeclarations) -> syn::Result<()> {
    let mut graph = Graph::new();
    let mut nodes = HashMap::new();

    fn get_or_add_node(nodes: &mut HashMap<syn::Type, NodeIndex>, graph: &mut Graph::<TypeNode, i32>, ty: &syn::Type) -> NodeIndex {
        *nodes.entry(ty.clone())
            .or_insert_with(|| {
                let node = TypeNode { state: ty.clone(), region: None };
                graph.add_node(node)
            })
    }

    for (ty, _) in &decl.states {
        get_or_add_node(&mut nodes, &mut graph, ty);
    }

    for transition in &decl.transitions {
        match transition.ty {
            crate::parse::FsmTransitionType::InternalTransition(_) => {}
            crate::parse::FsmTransitionType::SelfTransition(_) => {}
            crate::parse::FsmTransitionType::StateTransition(ref s) => {
                match (s.state_from.get_fsm_state(), s.state_to.get_fsm_state()) {
                    (Ok(from), Ok(to)) => {
                        let state_from = get_or_add_node(&mut nodes, &mut graph, &from.ty);
                        let state_to = get_or_add_node(&mut nodes, &mut graph, &to.ty);

                        graph.add_edge(state_from, state_to, 0);
                    },
                    (_, _) => ()
                }
            }
        }
    }

    let region_id = 0;
    {
        let start_node = get_or_add_node(&mut nodes, &mut graph, &decl.initial_state);
        let mut dfs = Dfs::new(&graph, start_node);
        while let Some(idx) = dfs.next(&graph) {
            graph[idx].region = Some(region_id);
        }
    }

    for node in graph.raw_nodes() {
        if node.weight.region == None {
            return Err(syn::Error::new(node.weight.state.span(), "Unreachable state! Add some transitions that will make this state reachable!"));
        }
    }
    
    Ok(())
}