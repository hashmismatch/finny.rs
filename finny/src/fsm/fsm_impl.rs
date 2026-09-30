use crate::{DispatchContext, FsmTimers, Inspect, lib::*};
use crate::{FsmBackend, FsmDispatch, FsmEvent, FsmEventQueue, FsmResult, FsmStates};

use super::FsmStateFactory;

/// The struct that holds the core context and state of the given Finny FSM. Doesn't include
/// environmental traits that can be changed at runtime.
pub struct FsmBackendImpl<F: FsmBackend> {
    pub context: <F as FsmBackend>::Context,
    pub states: <F as FsmBackend>::States,
    pub current_states: <<F as FsmBackend>::States as FsmStates<F>>::CurrentState
}

impl<F: FsmBackend> FsmBackendImpl<F> {
    pub fn new(context: <F as FsmBackend>::Context) -> FsmResult<Self> {

        let states = <<F as FsmBackend>::States>::new_state(&context)?;
        let current_states = <<<F as FsmBackend>::States as FsmStates<F>>::CurrentState>::default();

        let backend = FsmBackendImpl::<F> {
            context,
            states,
            current_states
        };

        Ok(backend)
    }
    
    pub fn get_context(&self) -> &<F as FsmBackend>::Context {
        &self.context
    }

    pub fn get_current_states(&self) -> <<F as FsmBackend>::States as FsmStates<F>>::CurrentState {
        self.current_states
    }

    pub fn get_state<S>(&self) -> &S
        where <F as FsmBackend>::States : AsRef<S>
    {
        self.states.as_ref()
    }
}

/// Serialized as `{ context, states, current_states }`, the current states have one entry for
/// each region.
#[cfg(feature = "serde")]
impl<F: FsmBackend> serde::Serialize for FsmBackendImpl<F>
    where <F as FsmBackend>::Context: serde::Serialize,
    <F as FsmBackend>::States: serde::Serialize,
    <<F as FsmBackend>::States as FsmStates<F>>::StateKind: serde::Serialize
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        let mut s = serializer.serialize_struct("FsmBackendImpl", 3)?;
        s.serialize_field("context", &self.context)?;
        s.serialize_field("states", &self.states)?;
        s.serialize_field("current_states", self.current_states.as_ref())?;
        s.end()
    }
}

/// Deserialized from the `{ context, states, current_states }` of the serialization. The current
/// states have to match the machine's regions. Restore the frontend from the deserialized backend
/// with `FsmFactory::restore`.
#[cfg(feature = "serde")]
impl<'de, F: FsmBackend> serde::Deserialize<'de> for FsmBackendImpl<F>
    where <F as FsmBackend>::Context: serde::Deserialize<'de>,
    <F as FsmBackend>::States: serde::Deserialize<'de>,
    <<F as FsmBackend>::States as FsmStates<F>>::StateKind: serde::Deserialize<'de>
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let repr = backend_serde::BackendRepr::<F, _, _>::deserialize(deserializer)?;

        Ok(FsmBackendImpl {
            context: repr.context,
            states: repr.states,
            current_states: repr.current_states.0
        })
    }
}

#[cfg(feature = "serde")]
mod backend_serde {
    use crate::{FsmBackend, FsmCurrentState, FsmStates, lib::*};
    use serde::de::{Deserialize, Deserializer, Error, SeqAccess, Visitor};

    type StateKind<F> = <<F as FsmBackend>::States as FsmStates<F>>::StateKind;
    type CurrentState<F> = <<F as FsmBackend>::States as FsmStates<F>>::CurrentState;

    #[derive(serde::Deserialize)]
    #[serde(rename = "FsmBackendImpl")]
    #[serde(bound(deserialize = "C: Deserialize<'de>, S: Deserialize<'de>, StateKind<F>: Deserialize<'de>"))]
    pub struct BackendRepr<F: FsmBackend, C, S> {
        pub context: C,
        pub states: S,
        pub current_states: CurrentStates<F>
    }

    /// The current state of each region, in the fixed-size array of the machine.
    pub struct CurrentStates<F: FsmBackend>(pub CurrentState<F>);

    impl<'de, F: FsmBackend> Deserialize<'de> for CurrentStates<F> where StateKind<F>: Deserialize<'de> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            deserializer.deserialize_seq(CurrentStatesVisitor::<F>(PhantomData))
        }
    }

    struct CurrentStatesVisitor<F>(PhantomData<F>);

    impl<'de, F: FsmBackend> Visitor<'de> for CurrentStatesVisitor<F> where StateKind<F>: Deserialize<'de> {
        type Value = CurrentStates<F>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("the current state of each region of the FSM")
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut current_states = CurrentState::<F>::default();
            let regions = current_states.as_ref().len();

            for (region, slot) in current_states.as_mut().iter_mut().enumerate() {
                let state: FsmCurrentState<StateKind<F>> = seq.next_element()?
                    .ok_or_else(|| A::Error::invalid_length(region, &self))?;
                if let FsmCurrentState::State(ref s) = state {
                    if !F::is_valid_current_state(region, s) {
                        return Err(A::Error::custom(format_args!("the state {:?} isn't in the region {} of the FSM", s, region)));
                    }
                }
                *slot = state;
            }

            if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
                return Err(A::Error::invalid_length(regions + 1, &self));
            }

            Ok(CurrentStates(current_states))
        }
    }
}

impl<F: FsmBackend> Deref for FsmBackendImpl<F> {
    type Target = <F as FsmBackend>::Context;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl<F: FsmBackend> DerefMut for FsmBackendImpl<F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}


/// The frontend of a state machine which also includes environmental services like queues
/// and inspection. The usual way to use the FSM.
pub struct FsmFrontend<F, Q, I, T> 
    where F: FsmBackend, Q: FsmEventQueue<F>, I: Inspect, T: FsmTimers<F>
{
    pub backend: FsmBackendImpl<F>,
    pub queue: Q,
    pub inspect: I,
    pub timers: T
}

impl<F, Q, I, T> FsmFrontend<F, Q, I, T>
    where F: FsmDispatch, Q: FsmEventQueue<F>, I: Inspect, T: FsmTimers<F>
{
    /// Start the FSM, initiates the transition to the initial state and runs the events
    /// enqueued by it to completition.
    pub fn start(&mut self) -> FsmResult<()> {
        self.dispatch_single_event(FsmEvent::Start)?;
        self.dispatch_queue()
    }

    /// Stop the FSM: exits the active states of all the regions, including their timers and
    /// sub-machines. The FSM can be started again.
    pub fn stop(&mut self) -> FsmResult<()> {
        self.dispatch_single_event(FsmEvent::Stop)?;
        self.dispatch_queue()
    }

    /// Dispatch any pending timer events into the queue, then run all the
    /// events from the queue until completition.
    pub fn dispatch_timer_events(&mut self) -> FsmResult<()> {
        loop {
            if let Some(timer_id) = self.timers.get_triggered_timer() {
                self.dispatch_single_event(FsmEvent::Timer(timer_id))?;
            } else {
                break;
            }
        }

        self.dispatch_queue()
    }

    /// Dispatch this event and run it to completition.
    pub fn dispatch<E>(&mut self, event: E) -> FsmResult<()>
        where E: Into<<F as FsmBackend>::Events>
    {
        let ev = event.into();
        let ev = FsmEvent::Event(ev);
        Self::dispatch_single_event(self, ev)?;

        self.dispatch_queue()
    }

    /// Dispatch only this event, do not run it to completition.
    pub fn dispatch_single_event(&mut self, event: FsmEvent<<F as FsmBackend>::Events, <F as FsmBackend>::Timers>) -> FsmResult<()> {
        let dispatch_ctx = DispatchContext {
            backend: &mut self.backend,
            inspect: &mut self.inspect,
            queue: &mut self.queue,
            timers: &mut self.timers
        };

        F::dispatch_event(dispatch_ctx, event)
    }

    /// Dispatch the entire event queue and run it to completition. The events that fail,
    /// usually because the current state doesn't handle them, are reported to the inspector
    /// with `Inspect::on_queued_event_error`.
    pub fn dispatch_queue(&mut self) -> FsmResult<()> {
        // A loop, the events enqueued by the dispatched events are appended to the same queue.
        while let Some(ev) = self.queue.dequeue() {
            if let Err(e) = self.dispatch_single_event(FsmEvent::Event(ev.clone())) {
                self.inspect.on_queued_event_error::<F>(&ev, &e);
            }
        }

        Ok(())
    }
}

impl<F, Q, I, T> Deref for FsmFrontend<F, Q, I, T>
    where F: FsmBackend, Q: FsmEventQueue<F>, I: Inspect, T: FsmTimers<F>
{
    type Target = FsmBackendImpl<F>;

    fn deref(&self) -> &Self::Target {
        &self.backend
    }
}

impl<F, Q, I, T> DerefMut for FsmFrontend<F, Q, I, T>
    where F: FsmBackend, Q: FsmEventQueue<F>, I: Inspect, T: FsmTimers<F>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.backend
    }
}