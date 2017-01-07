extern crate quote;
extern crate syn;

use fsm_def::*;

#[cfg(not(feature = "viz"))]
pub fn build_viz(fsm: &FsmDescription) -> quote::Tokens {
    quote! {}
}

#[cfg(feature = "viz")]
pub fn build_viz(fsm: &FsmDescription) -> quote::Tokens {

    let mut states_viz = String::new();
    let mut transitions_viz = String::new();

    //let mut viz_body = String::new();

    for region in &fsm.regions {

        if fsm.has_multiple_regions() {
            states_viz.push_str(&format!("\nsubgraph cluster_#prefix#_r_{id} {{\n label=\"Region {id}\"; style=dashed; \n", id=region.id));
        }

        states_viz.push_str(&format!(r##"#prefix#VizFsmInitial{} [ label = "Start", shape=diamond ];"##, region.id));
        transitions_viz.push_str(&format!("#prefix#VizFsmInitial{} -> #prefix#{};\n", region.id, ty_to_string(&region.initial_state_ty)));

        for state in &region.get_all_internal_states() {
            let mut n = ty_to_string(state);
            let mut label = n.clone();

            let mut style = String::new();
            if state == &region.initial_state_ty {
                style.push_str("style=\"filled\";");
            }
            if region.interrupt_states.iter().any(|x| &x.interrupt_state_ty == state) {
                style.push_str("shape=\"hexagon\"; fillcolor=crimson; style=filled;");
                label.push_str(" (Interrupt state)");
            }

            states_viz.push_str(&format!("#prefix#{name} [ label = \"{label}\" {style} ]; \n", name=n, label=label, style=style));
        }

        for sub in region.get_all_states().iter().filter(|ref x| fsm.is_submachine(x)) {
            let s = ty_to_string(&sub);
            states_viz.push_str(&format!("\nsubgraph cluster_{s} {{\n label=\"{s}\"; color = black;\n #sub_{s}# \n}}\n", s=s));
        }
        
        for transition in &region.transitions {
            let s_from = ty_to_string(&transition.source_state);
            let s_to = ty_to_string(&transition.target_state);
            let ev = ty_to_string(&transition.event);
            let ac = ty_to_string(&transition.action);

            
            let is_shallow_history = fsm.shallow_history_events.iter().find(|ref x| &x.event_ty == &transition.event && &x.target_state_ty == &transition.target_state).is_some();
            let history = if is_shallow_history {
                "(H)"
            } else {
                ""
            };
            
            let mut txt = format!("<FONT color=\"limegreen\">{}</FONT><BR/>/ <FONT color=\"crimson\">{}</FONT> {}", ev, ac, history);
            if region.interrupt_states.iter().any(|x| &x.interrupt_state_ty == &transition.source_state && x.resume_event_ty.iter().any(|y| y == &transition.event)) {
                txt.push_str("<BR/> Resume event");
            }

            let mut additional = String::new();

            if transition.transition_type == TransitionType::Internal {
                additional.push_str(&", style=dotted, dirType=none");
            }

            if fsm.is_submachine(&transition.source_state) {
                transitions_viz.push_str(&format!("{}VizFsmInitial{} -> #prefix#{} [ label=<{}>, ltail=cluster_{} {} ];\n", s_from, region.id, s_to, txt, s_from, additional));
            } else if fsm.is_submachine(&transition.target_state) {
                if is_shallow_history {
                    transitions_viz.push_str(&format!("#prefix#{} -> {}VizFsmInitial{} [ label=<{}>, lhead=cluster_{} {} ];\n", s_from, s_to, region.id, txt, s_to, additional)); 
                } else {
                    transitions_viz.push_str(&format!("#prefix#{} -> {}VizFsmInitial{} [ label=<{}> {} ];\n", s_from, s_to, region.id, txt, additional));
                }
            } else {
                transitions_viz.push_str(&format!("#prefix#{} -> #prefix#{} [ label = <{}> {} ];\n", s_from, s_to, txt, additional));
            }
        }

        if fsm.has_multiple_regions() {
            states_viz.push_str("\n}");
        }
        
    }
    


    let mut subs = quote::Tokens::new();
    for sub in fsm.get_all_states().iter().filter(|ref x| fsm.is_submachine(x)) {
        let s = ty_to_string(&sub);        
        let a = format!("#sub_{}#", s);        
        subs.append(quote! {
            t = t.replace(#a, &#sub::viz_body(#s));
        }.as_str());
    }
    
    quote! {
        
        fn viz_body(prefix: &str) -> String {
            let mut t: String = #states_viz.into();
            t.push_str(& #transitions_viz);            
            t = t.replace("#prefix#", prefix);
            #subs
            t
        }

        fn viz_digraph() -> String {
            format!(r#"digraph G {{
                compound=true;
                layout=dot;
                overlap = false;
                splines = true;

                /* {{rank=source VizFsmInitial}} */
                {}
            }}"#, Self::viz_body(""))
        }
    }
    
}

#[cfg(not(feature = "viz"))]
pub fn build_test_viz_build(fsm: &FsmDescription) -> quote::Tokens {
    quote! { }
}

#[cfg(feature = "viz")]
pub fn build_test_viz_build(fsm: &FsmDescription) -> quote::Tokens {
    let fn_name = fsm.get_build_viz_fn();
    let ty = fsm.get_fsm_ty_inline();
    let ref ty_str = fsm.name;


    let output_file = quote! {
        let f: String = {                
            let f: Vec<_> = file!().split('.').collect();
            
            if let Some(i) = f.iter().rposition(|&x| x == "rs") {
                let l: String = f[i-1].to_string();
                let s: Vec<_> = l.split(|c| c == '/' || c == '\\' || c == ':').collect();
                s.iter().last().unwrap().to_string()
            } else {
                "file".to_string()
            }
        };
    };
    

    quote! {
        #[test]
        #[cfg(test)]
        fn #fn_name () {
            
            #output_file
            let output_file = format!("{}_{}.gv", f, #ty_str);

            use std::io::prelude::*;
            use std::fs::File;

            let d = #ty::viz_digraph();

            let mut f = File::create(&output_file).unwrap();
            f.write_all(d.as_bytes()).unwrap();
        }
    }
}