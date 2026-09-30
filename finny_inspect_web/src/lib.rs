//! # Finny web inspector
//!
//! A web frontend for inspecting the [Finny](https://docs.rs/finny) state machines while they
//! run: a statechart of the machine with its current states, the history of the dispatched
//! events with their traces (guards, exits, actions, entries) and the values of the context,
//! the states and the events.
//!
//! The [`Inspector`] keeps a snapshot of the machine after each event that it has fully
//! handled, the last 100 by default, so the frontend can trace the history back when it
//! connects.
//!
//! ```no_run
//! use finny::{FsmEventQueueVec, FsmFactory, FsmResult, decl::{BuiltFsm, FsmBuilder}, finny_fsm, timers::std::TimersStd};
//! use finny_inspect_web::{Inspector, InspectorConfig};
//! use serde::Serialize;
//!
//! #[derive(Default, Serialize)]
//! pub struct Ctx { count: usize }
//! #[derive(Default, Serialize)]
//! pub struct Idle;
//! #[derive(Clone, Debug, Serialize)]
//! pub struct Poke;
//!
//! #[finny_fsm]
//! fn my_fsm(mut fsm: FsmBuilder<MyFsm, Ctx>) -> BuiltFsm {
//!     // Opt into the serialization of the context, states and events.
//!     fsm.serde();
//!     fsm.initial_state::<Idle>();
//!     fsm.state::<Idle>()
//!         .on_event::<Poke>()
//!         .self_transition()
//!         .action(|_, ctx, _| { ctx.count += 1; });
//!     fsm.build()
//! }
//!
//! fn main() -> FsmResult<()> {
//!     let inspector = Inspector::new(InspectorConfig::default());
//!     let addr = inspector.spawn("127.0.0.1:7878").expect("Failed to start the inspector");
//!     println!("http://{}", addr);
//!
//!     let mut fsm = MyFsm::new_with(Ctx::default(), FsmEventQueueVec::new(),
//!         inspector.attach::<MyFsm>("main"), TimersStd::new())?;
//!     fsm.start()?;
//!     fsm.dispatch(Poke)?;
//!     Ok(())
//! }
//! ```
//!
//! Chain it with other inspectors using `finny::inspect::chain::InspectChain`. FSMs that don't
//! opt in with `fsm.serde()` are inspected too, just without the values.

mod assets;
mod config;
mod inspect;
mod registry;
mod server;
pub mod snapshot;

pub use config::{ASSETS_DIR_ENV, InspectorConfig};
pub use inspect::InspectWeb;
pub use registry::{InstanceSummary, Inspector};
