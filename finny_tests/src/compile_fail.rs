//! Definitions that the code generator or the compiler have to reject.
//!
//! The actions of an async FSM have to be async closures:
//!
//! ```compile_fail
//! use finny::{finny_fsm, decl::{BuiltFsm, FsmAsyncBuilder}};
//! #[derive(Default)]
//! pub struct StateA;
//!
//! #[finny_fsm]
//! fn build(mut fsm: FsmAsyncBuilder<Machine, ()>) -> BuiltFsm {
//!     fsm.initial_state::<StateA>();
//!     fsm.state::<StateA>().on_entry(|_, _| { });
//!     fsm.build()
//! }
//! # fn main() {}
//! ```
//!
//! Not closures that return an async block, it couldn't borrow the arguments:
//!
//! ```compile_fail
//! use finny::{finny_fsm, decl::{BuiltFsm, FsmAsyncBuilder}};
//! #[derive(Default)]
//! pub struct StateA;
//!
//! #[finny_fsm]
//! fn build(mut fsm: FsmAsyncBuilder<Machine, ()>) -> BuiltFsm {
//!     fsm.initial_state::<StateA>();
//!     fsm.state::<StateA>().on_entry(|_, _| async move { });
//!     fsm.build()
//! }
//! # fn main() {}
//! ```
//!
//! Guards are always synchronous:
//!
//! ```compile_fail
//! use finny::{finny_fsm, decl::{BuiltFsm, FsmAsyncBuilder}};
//! #[derive(Default)]
//! pub struct StateA;
//! #[derive(Clone)]
//! pub struct Event;
//!
//! #[finny_fsm]
//! fn build(mut fsm: FsmAsyncBuilder<Machine, ()>) -> BuiltFsm {
//!     fsm.initial_state::<StateA>();
//!     fsm.state::<StateA>().on_event::<Event>().internal_transition().guard(async |_, _, _| true);
//!     fsm.build()
//! }
//! # fn main() {}
//! ```
//!
//! Sync FSMs can't have async actions:
//!
//! ```compile_fail
//! use finny::{finny_fsm, decl::{BuiltFsm, FsmBuilder}};
//! #[derive(Default)]
//! pub struct StateA;
//!
//! #[finny_fsm]
//! fn build(mut fsm: FsmBuilder<Machine, ()>) -> BuiltFsm {
//!     fsm.initial_state::<StateA>();
//!     fsm.state::<StateA>().on_entry(async |_, _| { });
//!     fsm.build()
//! }
//! # fn main() {}
//! ```
//!
//! Nor concurrent regions:
//!
//! ```compile_fail
//! use finny::{finny_fsm, decl::{BuiltFsm, FsmBuilder}};
//! #[derive(Default)]
//! pub struct StateA;
//!
//! #[finny_fsm]
//! fn build(mut fsm: FsmBuilder<Machine, ()>) -> BuiltFsm {
//!     fsm.concurrent_regions();
//!     fsm.initial_state::<StateA>();
//!     fsm.state::<StateA>();
//!     fsm.build()
//! }
//! # fn main() {}
//! ```
//!
//! Transitions are tried in the order of their declaration. A transition that follows an
//! unguarded one for the same state and event could never be taken:
//!
//! ```compile_fail
//! use finny::{finny_fsm, decl::{BuiltFsm, FsmBuilder}};
//! #[derive(Default)]
//! pub struct StateA;
//! #[derive(Default)]
//! pub struct StateB;
//! #[derive(Default)]
//! pub struct StateC;
//! #[derive(Clone)]
//! pub struct Event;
//!
//! #[finny_fsm]
//! fn build(mut fsm: FsmBuilder<Machine, ()>) -> BuiltFsm {
//!     fsm.initial_state::<StateA>();
//!     fsm.state::<StateA>().on_event::<Event>().transition_to::<StateB>();
//!     fsm.state::<StateA>().on_event::<Event>().transition_to::<StateC>().guard(|_, _, _| true);
//!     fsm.state::<StateB>();
//!     fsm.state::<StateC>();
//!     fsm.build()
//! }
//! # fn main() {}
//! ```
//!
//! A transition into the same state has to be a self or an internal transition:
//!
//! ```compile_fail
//! use finny::{finny_fsm, decl::{BuiltFsm, FsmBuilder}};
//! #[derive(Default)]
//! pub struct StateA;
//! #[derive(Clone)]
//! pub struct Event;
//!
//! #[finny_fsm]
//! fn build(mut fsm: FsmBuilder<Machine, ()>) -> BuiltFsm {
//!     fsm.initial_state::<StateA>();
//!     fsm.state::<StateA>().on_event::<Event>().transition_to::<StateA>();
//!     fsm.build()
//! }
//! # fn main() {}
//! ```
//!
//! Each region has its own initial state:
//!
//! ```compile_fail
//! use finny::{finny_fsm, decl::{BuiltFsm, FsmBuilder}};
//! #[derive(Default)]
//! pub struct StateA;
//!
//! #[finny_fsm]
//! fn build(mut fsm: FsmBuilder<Machine, ()>) -> BuiltFsm {
//!     fsm.initial_states::<(StateA, StateA)>();
//!     fsm.state::<StateA>();
//!     fsm.build()
//! }
//! # fn main() {}
//! ```
//!
//! And the regions can't have transitions between them:
//!
//! ```compile_fail
//! use finny::{finny_fsm, decl::{BuiltFsm, FsmBuilder}};
//! #[derive(Default)]
//! pub struct StateA;
//! #[derive(Default)]
//! pub struct StateB;
//! #[derive(Clone)]
//! pub struct Event;
//!
//! #[finny_fsm]
//! fn build(mut fsm: FsmBuilder<Machine, ()>) -> BuiltFsm {
//!     fsm.initial_states::<(StateA, StateB)>();
//!     fsm.state::<StateA>().on_event::<Event>().transition_to::<StateB>();
//!     fsm.state::<StateB>();
//!     fsm.build()
//! }
//! # fn main() {}
//! ```
//!
//! An async FSM whose context isn't `Sync` can't be moved to another thread, its `Arc` isn't `Send`:
//!
//! ```compile_fail
//! use std::cell::Cell;
//! use finny::{FsmAsyncFactory, finny_fsm, decl::{BuiltFsm, FsmAsyncBuilder}};
//!
//! #[derive(Default)]
//! pub struct Ctx { counter: Cell<usize> }
//! #[derive(Default)]
//! pub struct StateA;
//!
//! #[finny_fsm]
//! fn build(mut fsm: FsmAsyncBuilder<Machine, Ctx>) -> BuiltFsm {
//!     fsm.initial_state::<StateA>();
//!     fsm.state::<StateA>().on_entry(async |_, ctx| { ctx.counter.set(1); });
//!     fsm.build()
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut fsm = Machine::new(Ctx::default()).unwrap();
//!     tokio::spawn(async move { fsm.start().await }).await.unwrap().unwrap();
//! }
//! ```
//!
//! The same FSM runs fine on the current task:
//!
//! ```
//! use std::cell::Cell;
//! use finny::{FsmAsyncFactory, finny_fsm, decl::{BuiltFsm, FsmAsyncBuilder}};
//!
//! #[derive(Default)]
//! pub struct Ctx { counter: Cell<usize> }
//! #[derive(Default)]
//! pub struct StateA;
//!
//! #[finny_fsm]
//! fn build(mut fsm: FsmAsyncBuilder<Machine, Ctx>) -> BuiltFsm {
//!     fsm.initial_state::<StateA>();
//!     fsm.state::<StateA>().on_entry(async |_, ctx| { ctx.counter.set(1); });
//!     fsm.build()
//! }
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() {
//!     let mut fsm = Machine::new(Ctx::default()).unwrap();
//!     fsm.start().await.unwrap();
//!     assert_eq!(1, fsm.counter.get());
//! }
//! ```
//!
//! An FSM that opts into the serialization needs `Serialize` on its states:
//!
//! ```compile_fail
//! use finny::{finny_fsm, decl::{BuiltFsm, FsmBuilder}};
//! #[derive(Default)]
//! pub struct StateA;
//!
//! #[finny_fsm]
//! fn build(mut fsm: FsmBuilder<Machine, ()>) -> BuiltFsm {
//!     fsm.serde();
//!     fsm.initial_state::<StateA>();
//!     fsm.state::<StateA>();
//!     fsm.build()
//! }
//! # fn main() {}
//! ```
//!
//! With them, it compiles:
//!
//! ```
//! use finny::{finny_fsm, decl::{BuiltFsm, FsmBuilder}};
//! #[derive(Default, serde::Serialize)]
//! pub struct StateA;
//!
//! #[finny_fsm]
//! fn build(mut fsm: FsmBuilder<Machine, ()>) -> BuiltFsm {
//!     fsm.serde();
//!     fsm.initial_state::<StateA>();
//!     fsm.state::<StateA>();
//!     fsm.build()
//! }
//! # fn main() {}
//! ```
