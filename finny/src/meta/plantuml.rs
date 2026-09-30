//! Renders the machine's description as a PlantUML state diagram.

use alloc::string::String;
use core::fmt::Write;

use super::{FsmInfo, TransitionKindInfo};

/// A complete PlantUML document, `@startuml` .. `@enduml`.
pub fn to_plantuml(fsm: &FsmInfo) -> String {
    let mut output = String::new();
    let _ = writeln!(output, "@startuml {}", fsm.id);
    write_fsm(&mut output, fsm, "").expect("Writing into a String can't fail");
    let _ = writeln!(output, "@enduml");
    output
}

/// The states and the transitions of the machine, without the document's header. The regions
/// are separated with `--`.
pub fn to_plantuml_inner(fsm: &FsmInfo) -> String {
    let mut output = String::new();
    write_fsm(&mut output, fsm, "").expect("Writing into a String can't fail");
    output
}

fn write_fsm(output: &mut String, fsm: &FsmInfo, indent: &str) -> core::fmt::Result {
    for (i, region) in fsm.regions.iter().enumerate() {
        if i > 0 {
            writeln!(output, "{}--", indent)?;
        }

        for state in &region.states {
            match state.sub_machine {
                Some(ref sub) => {
                    writeln!(output, "{}state {} {{", indent, state.id)?;
                    let nested = alloc::format!("{}  ", indent);
                    write_fsm(output, sub, &nested)?;
                    writeln!(output, "{}}}", indent)?;
                },
                None => {
                    writeln!(output, "{}state {}", indent, state.id)?;
                }
            }

            for timer in &state.timers {
                writeln!(output, "{}state {} : Timer {}", indent, state.id, timer.id)?;
            }
        }

        for transition in &region.transitions {
            let event = transition.event.label();
            let guard = if transition.has_guard { " [guard]" } else { "" };
            match &transition.kind {
                TransitionKindInfo::Normal { from, to } => {
                    let from = from.as_deref().unwrap_or("[*]");
                    let to = to.as_deref().unwrap_or("[*]");
                    writeln!(output, "{}{} --> {} : {}{}", indent, from, to, event, guard)?;
                },
                TransitionKindInfo::SelfTransition { state } => {
                    writeln!(output, "{}{} --> {} : {}{} (Self)", indent, state, state, event, guard)?;
                },
                TransitionKindInfo::Internal { state } => {
                    writeln!(output, "{}state {} : {}{} (Internal)", indent, state, event, guard)?;
                }
            }
        }
    }

    Ok(())
}
