# Finny - Finite State Machines for Rust

![Build](https://github.com/hashmismatch/finny.rs/workflows/Build/badge.svg)

## Features
* Declarative, builder API with procedural macros that generate the necessary boilerplate
* Compile-time transition graph validation
* No allocations required
* Support for generics within the shared context
* Transition guards and actions
* `no_std` support

## Example

```rust
extern crate finny;
use finny::{finny_fsm, FsmFactory, FsmResult, decl::{BuiltFsm, FsmBuilder}};

// The context is shared between all guards, actions and transitions. Generics are supported here!
#[derive(Default)]
pub struct MyContext { val: u32 }
// The states are plain structs.
#[derive(Default)]
pub struct MyStateA { n: usize }
#[derive(Default)]
pub struct MyStateB;
// The events are also plain structs. They can have fields.
pub struct MyEvent;

// The FSM is generated by a procedural macro
#[finny_fsm]
fn my_fsm(mut fsm: FsmBuilder<MyFsm, MyContext>) -> BuiltFsm {
    // The FSM is describe using a builder-style API
    fsm.state::<MyStateA>()
       .on_entry(|state, ctx| {
           state.n += 1;
           ctx.context.val += 1;
        });
    fsm.state::<MyStateB>();
    fsm.on_event::<MyEvent>()
       .transition_from::<MyStateA>().to::<MyStateB>()
       .guard(|_ev, ctx| { ctx.context.val > 0 })
       .action(|_ev, ctx, state_a, state_b| { ctx.context.val += 1; });
    fsm.initial_state::<MyStateA>();
    fsm.build()
}

// The FSM is built and tested.
fn main() -> FsmResult<()> {
    let mut fsm = MyFsm::new(MyContext::default())?;
    assert_eq!(0, fsm.val);
    fsm.start()?;
    let state_a: &MyStateA = fsm.get_state();
    assert_eq!(1, state_a.n);
    assert_eq!(1, fsm.val);
    fsm.dispatch(MyEvent)?;
    assert_eq!(2, fsm.val);
    Ok(())
}
```