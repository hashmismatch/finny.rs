extern crate quote;
extern crate syn;

use fsm_def::*;

#[cfg(not(feature = "viz"))]
pub fn build_viz(fsm: &FsmDescription) -> quote::Tokens {
    quote! {}
}

#[cfg(feature = "viz")]
pub fn build_viz(fsm: &FsmDescription) -> quote::Tokens {
    use std::fmt::Write;
    let js_lib = include_str!("viz_fsm.js");
    let template = include_str!("viz.html");

    let mut subs = quote::Tokens::new();

    let mut out = String::new();
    writeln!(out, "var f = newFsm(cy, '{}', '##parent##'); var fsm_{} = f;", fsm.name, fsm.name);

    for region in &fsm.regions {
        let var_region = format!("{}_region_{}", fsm.name, region.id);
        writeln!(out, r#"var {} = f.add_region("Region {}");"#, var_region, region.id);

        let initial_name = ty_to_string(&region.initial_state_ty);
        writeln!(out, r#"var state_{} = {}.add_initial_state("{}");"#, initial_name, var_region, initial_name);
        for state in &region.get_all_internal_states() {
            if state == &region.initial_state_ty {
                continue;
            }

            let is_initial_state = state == &region.initial_state_ty;
            let is_interrupt_state = region.interrupt_states.iter().any(|x| &x.interrupt_state_ty == state);

            let info = format!(r#"{{ is_initial_state: {:?}, is_interrupt_state: {:?} }}"#, is_initial_state, is_interrupt_state);

            let name = ty_to_string(state);
            writeln!(out, r#"var state_{} = {}.add_state("{}", {});"#, name, var_region, name, info);
        }

        for sub in region.get_all_states().iter().filter(|ref x| fsm.is_submachine(x)) {
            let s = ty_to_string(&sub);

            // submachines
            writeln!(out, "// submachine {} start", s);
            writeln!(out, "// ##SUB_{}##", s);
            writeln!(out, "// submachine {} end", s);
            
            {
                let a = format!("// ##SUB_{}##", s);
                let p = format!("fsm_{}", fsm.name);
                subs.append(quote! {
                    let t = t.replace(#a, &#sub::viz_cytoscape_fsm(#p));
                }.as_str());
            }
        }

        for transition in &region.transitions {
            let s_from = ty_to_string(&transition.source_state);
            let s_to = ty_to_string(&transition.target_state);
            let ev = ty_to_string(&transition.event);
            let ac = ty_to_string(&transition.action);

            
            let is_shallow_history = fsm.shallow_history_events.iter().find(|ref x| &x.event_ty == &transition.event && &x.target_state_ty == &transition.target_state).is_some();
            let is_resume_event = region.interrupt_states.iter().any(|x| &x.interrupt_state_ty == &transition.source_state && x.resume_event_ty.iter().any(|y| y == &transition.event));
            let is_internal = transition.transition_type == TransitionType::Internal;
            let is_anonymous = transition.is_anonymous_transition();

            let guard_json = match transition.guard {
                Some(ref g) => ty_to_string(g),
                None => "".into()
            };
            let data = {
                let ac = if ac == "NoAction" { "".into() } else { ac };
                format!("{{ event: '{}', action: '{}', guard: '{}', transition_type: '{}', is_anonymous: {:?}, shallow_history: {:?}, resume_event: {:?} }}", ev, ac, guard_json, transition.transition_type, is_anonymous, is_shallow_history, is_resume_event)
            };

            let (from, to) = {
                if fsm.is_submachine(&transition.source_state) {
                    (format!("fsm_{}", s_from), format!("state_{}", s_to))
                } else if fsm.is_submachine(&transition.target_state) {
                    
                    if is_shallow_history {
                        (format!("state_{}", s_from), format!("fsm_{}", s_to))
                    } else {
                        writeln!(out, r#"fsm_{}.add_transition_to_start({}, {});"#, s_to, format!("state_{}", s_from), data);
                        continue;
                    }
                } else {
                    (format!("state_{}", s_from), format!("state_{}", s_to))
                }
            };

            writeln!(out, r#"{}.add_transition({}, {}, {});"#, var_region, from, to, data);
        }
    }

    

    quote! {
        fn viz_cytoscape_fsm(parent: &str) -> String {            
            let t = (#out).replace("##parent##", parent);
            #subs
            t
        }

        fn viz_cytoscape() -> String {
            let mut complete_js = #js_lib.to_string();
            complete_js.push_str(&format!("\nvar cy = init_cy_fsm();\n"));
            complete_js.push_str(&Self::viz_cytoscape_fsm(""));
            complete_js.push_str(&format!("\n f.run_layout(); \n"));
            #template.replace("// ##VIZ_FSM###", &complete_js)
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
    let fn_name_docs = fsm.get_build_viz_docs_fn();    
    let ty = fsm.get_fsm_ty_inline();
    let ref ty_str = fsm.name;

    let js_lib = include_str!("viz_fsm.js");
    let fs_html = include_str!("viz.html");


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

    let build_inline_docs = match () {
        #[cfg(not(feature = "viz_docs"))]
        () => quote! {},
        #[cfg(feature = "viz_docs")]
        () => quote! {
            #[test]
            #[cfg(test)]
            fn #fn_name_docs () {
                use std::io::prelude::*;
                use std::fs;

                #output_file

                let rust_module_path = #ty::module_path();
                let full_module_path = rust_module_path.replace("::", "/");
                let crate_path = rust_module_path.split("::").next().unwrap();

                let dir = format!("./target/doc/{}/", crate_path);
                fs::create_dir_all(&dir);

                let output_js = format!("fsm_viz_{}.js", #ty_str);
                let output_file = format!("{}{}", dir, output_js);

                let d = #ty::viz_cytoscape_fsm("");
                let d = format!("function viz_fsm_body(cy) {{\n{}\n return f; \n}}\n", d);

                let mut f = fs::File::create(&output_file).unwrap();
                f.write_all(d.as_bytes()).unwrap();

                {
                    let d = #fs_html.replace("##VIZ_JS_FSM##", &output_js);
                    let output_html = format!("{}fsm_viz_{}.html", dir, #ty_str);
                    
                    let mut f = fs::File::create(&output_html).unwrap();
                    f.write_all(d.as_bytes()).unwrap();
                }

                let mut f = fs::File::create(&format!("{}viz_fsm.js", dir)).unwrap();
                f.write_all(&#js_lib.as_bytes()).unwrap();
            }
        }
    };
    

    quote! {
        #[test]
        #[cfg(test)]
        fn #fn_name () {
            
            #output_file
            let output_file = format!("{}_{}.html", f, #ty_str);

            use std::io::prelude::*;
            use std::fs::File;

            let d = #ty::viz_cytoscape();

            let mut f = File::create(&output_file).unwrap();
            f.write_all(d.as_bytes()).unwrap();
        }

        #build_inline_docs
    }
}