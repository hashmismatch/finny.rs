use finny::{FsmFactory, FsmResult, decl::{BuiltFsm, FsmBuilder}, finny_fsm};

#[derive(Default, Debug)]
struct State {

}

#[finny_fsm]
fn build_fsm(mut fsm: FsmBuilder<MiniMachine, ()>) -> BuiltFsm {
    fsm.initial_state::<State>();
    fsm.state::<State>();
    fsm.build()
}
