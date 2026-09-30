use crate::lib::*;
use std::sync::Arc;

use crate::FsmBackend;
use super::{AsyncMode, BuiltFsm, FsmStateBuilder, FsmSubMachineBuilder};

/// The builder-API for defining an async Finny state machine. The state actions and
/// transition actions are async closures (`async |..| { .. }`), guards and timer setups
/// stay synchronous.
///
/// The FSM's context is shared through an `Arc`: actions receive it through the
/// `EventContext` and can clone it into spawned tasks. Use interior mutability
/// (atomics, mutexes) for the parts of the context that the actions modify.
///
/// The generated FSM implements `FsmAsyncFactory` and is driven through an `FsmAsyncFrontend`:
/// either by awaiting `dispatch` directly, or by running its event loop with `run`, which
/// dispatches the events from a tokio channel and the triggered timers.
///
/// Sub-machines of an async FSM have to be async FSMs as well.
///
/// ```rust
/// use std::{sync::atomic::{AtomicU32, Ordering}, time::Duration};
/// use finny::{finny_fsm, FsmAsyncFactory, FsmResult, decl::{BuiltFsm, FsmAsyncBuilder}};
///
/// // Shared through an `Arc`, the actions modify it through atomics.
/// #[derive(Default)]
/// pub struct MyContext { val: AtomicU32 }
/// #[derive(Default)]
/// pub struct MyStateA;
/// #[derive(Default)]
/// pub struct MyStateB;
/// #[derive(Clone)]
/// pub struct MyEvent;
/// #[derive(Clone)]
/// pub struct Tick;
///
/// #[finny_fsm]
/// fn my_fsm(mut fsm: FsmAsyncBuilder<MyFsm, MyContext>) -> BuiltFsm {
///     fsm.state::<MyStateA>()
///        .on_entry(async |_state, ctx| {
///            tokio::time::sleep(Duration::from_millis(10)).await;
///            ctx.val.fetch_add(1, Ordering::SeqCst);
///        })
///        .on_event::<MyEvent>()
///        .transition_to::<MyStateB>()
///        // guards stay synchronous
///        .guard(|_ev, ctx, _states| ctx.val.load(Ordering::SeqCst) > 0)
///        .action(async |_ev, ctx, _state_a, _state_b| { ctx.val.fetch_add(1, Ordering::SeqCst); });
///
///     fsm.state::<MyStateB>()
///        .on_entry_start_timer(|_ctx, timer| {
///            timer.timeout = Duration::from_millis(100);
///            timer.renew = true;
///        }, |_ctx, _state| Some(Tick.into()));
///
///     fsm.state::<MyStateB>()
///        .on_event::<Tick>()
///        .internal_transition()
///        .action(async |_ev, ctx, _state_b| { ctx.val.fetch_add(1, Ordering::SeqCst); });
///
///     fsm.initial_state::<MyStateA>();
///     fsm.build()
/// }
///
/// #[tokio::main(flavor = "current_thread", start_paused = true)]
/// async fn main() -> FsmResult<()> {
///     let mut fsm = MyFsm::new(MyContext::default())?;
///     fsm.start().await?;
///     assert_eq!(1, fsm.val.load(Ordering::SeqCst));
///     fsm.dispatch(MyEvent).await?;
///     assert_eq!(2, fsm.val.load(Ordering::SeqCst));
///
///     // The event loop dispatches the events sent from other tasks and the timers,
///     // until all the senders are dropped.
///     let (tx, mut rx) = tokio::sync::mpsc::channel(16);
///     tokio::spawn(async move {
///         tokio::time::sleep(Duration::from_millis(250)).await;
///         drop(tx);
///     });
///     fsm.run(&mut rx).await?;
///
///     // the timer triggered twice
///     assert_eq!(4, fsm.val.load(Ordering::SeqCst));
///     Ok(())
/// }
/// ```
pub struct FsmAsyncBuilder<TFsm, TContext> {
	pub _fsm: PhantomData<TFsm>,
	pub _context: PhantomData<TContext>
}

impl<TFsm, TContext> Default for FsmAsyncBuilder<TFsm, TContext> {
	fn default() -> Self {
		Self {
			_fsm: PhantomData::default(),
			_context: PhantomData::default()
		}
	}
}

impl<TFsm, TContext> FsmAsyncBuilder<TFsm, TContext>
	where TFsm: FsmBackend<Context = Arc<TContext>>
{
	/// Sets the initial state of the state machine. Required!
	pub fn initial_state<TSTate>(&mut self) {

	}

	/// Defines multiple initial states for multiple regions of the FSM. The type has to be a tuple
	/// of the initial states for each region.
	///
	/// Example : `fsm.initial_states<(StateA, StateX)>()`
	pub fn initial_states<TStates>(&mut self) {

	}

	/// Require the `Debug` trait on the Events.
	pub fn events_debug(&mut self) {

	}

	/// Run the actions of the FSM's regions concurrently, within the same task. Only has an effect
	/// on FSMs with multiple regions.
	///
	/// All the regions first select their transitions, evaluating the guards against the state
	/// before the event was dispatched. Then the selected transitions of all the regions are
	/// executed concurrently. Events enqueued by the actions are appended to the queue in the
	/// order of the regions. Each region's actions get their own clone of the context's `Arc`;
	/// an action that replaces it replaces the FSM's context once all the regions are done (the
	/// last region's replacement wins).
	pub fn concurrent_regions(&mut self) {

	}

	/// Adds some information about a state.
	pub fn state<TState>(&mut self) -> FsmStateBuilder<TFsm, Arc<TContext>, TState, AsyncMode> {
		FsmStateBuilder::new()
	}

	/// Adds a sub machine. The sub machine has to be an async FSM as well.
	pub fn sub_machine<TSubFsm>(&mut self) -> FsmSubMachineBuilder<TFsm, Arc<TContext>, TSubFsm, AsyncMode>
		where TSubFsm: FsmBackend
	{
		FsmSubMachineBuilder::new()
	}

	/// Builds the final machine. Has to be returned from the definition function.
	pub fn build(self) -> BuiltFsm {
		BuiltFsm
	}
}
